use std::{net::SocketAddr, sync::Arc};

use http::{Response, StatusCode, Uri};
use hyper::{body::Incoming, Request};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, sync::RwLock};

use crate::gateway::{
    bodies::{pinned_body::box_pinned_body, single_chunk_response_body, BoxBody},
    filters::{Filter, Filterable},
    route::Route,
};

/// Main service entry point for each request.
pub async fn responder(
    mut req: Request<Incoming>,
    routes: Arc<RwLock<Vec<Route>>>,
    remote_addr: SocketAddr,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Optionally store remote_addr in request.extensions
    req.extensions_mut().insert(remote_addr);

    let routes_guard = routes.read().await;
    if let Some(route) = routes_guard.iter().find(|route| route.matches(&req)) {
        // Apply filters
        match apply_filters(&route.filters, req).await {
            Ok(filtered_req) => forward_request(filtered_req, &route.destination).await,
            Err(_e) => {
                // If filter application fails, produce 500
                let body = single_chunk_response_body("Filter application failed");
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(body)
                    .unwrap())
            }
        }
    } else {
        // No route matched => 404
        let body = single_chunk_response_body("No matching route found");
        Ok(Response::builder().status(StatusCode::NOT_FOUND).body(body).unwrap())
    }
}

/// Apply filters to the incoming request.
async fn apply_filters(
    filters: &[Filter],
    mut req: Request<Incoming>,
) -> Result<Request<Incoming>, hyper::Error> {
    // Example: Each filter might manipulate headers, URIs, etc.
    for filter in filters {
        req = filter.apply(req).await?;
    }
    Ok(req)
}

async fn forward_request(
    req: Request<Incoming>,
    destination: &str,
) -> Result<Response<BoxBody>, hyper::Error> {
    let uri = match destination.parse::<Uri>() {
        Ok(u) => u,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: invalid URI"))
                .unwrap());
        }
    };

    let host = match uri.host() {
        Some(h) => h,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: missing host"))
                .unwrap());
        }
    };

    let port = uri.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);

    // Handle I/O errors with a 502.
    let stream = match TcpStream::connect(address).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Connection error: {e}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: cannot connect"))
                .unwrap());
        }
    };

    let io = TokioIo::new(stream);

    let (mut sender, conn) = match hyper::client::conn::http1::handshake(io).await {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("Handshake error: {e}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: handshake failed"))
                .unwrap());
        }
    };

    // Drive the connection in a background task
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Connection failed: {:?}", err);
        }
    });

    // Send request to the remote server
    let response = match sender.send_request(req).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Forward request error: {e}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: request failed"))
                .unwrap());
        }
    };

    // The remote server's response body is usually `Incoming` with `Data=Bytes` and `Error=hyper::Error`.
    // Just pin it, turning it into `Box<dyn Body<...> + Send>`.
    Ok(response.map(|b| box_pinned_body(b)))
}
