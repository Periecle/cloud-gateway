use crate::gateway::{
    bodies::{pinned_body::box_pinned_body, single_chunk_response_body, BoxBody},
    filters::{Filter, Filterable},
    route::Route,
};
use crate::gateway_metrics::inc_requests_total;
use crate::gateway_metrics::inc_responses_total;
use crate::gateway_metrics::record_filter_duration;
use http::{Response, StatusCode, Uri};
use hyper::{body::Incoming, Request};
use hyper_util::rt::TokioIo;
use log::{info, warn};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock};

/// Main service entry point for each request.
pub async fn responder(
    mut req: Request<Incoming>,
    routes: Arc<RwLock<Vec<Route>>>,
    remote_addr: SocketAddr,
) -> Result<Response<BoxBody>, hyper::Error> {
    let req_start_time = std::time::Instant::now();
    let method = req.method().to_string(); // Convert to String for static lifetime
    let uri_path = req.uri().path().to_string(); // Convert to String for static lifetime
    info!("Handling request: {method} {uri_path}");
    inc_requests_total();
    req.extensions_mut().insert(remote_addr);

    let routes_guard = routes.read().await;
    if let Some(route) = routes_guard.iter().find(|route| route.matches(&req)) {
        info!("Route matched: {:?}", route);
        // Apply filters
        match apply_filters(&route.filters, req).await {
            Ok(filtered_req) => {
                let response = forward_request(filtered_req, &route.destination).await?;
                let duration = req_start_time.elapsed().as_secs_f64();
                info!("Request processed in {:.3} ms", duration * 1000.0);
                inc_responses_total();
                Ok(response)
            }
            Err(_e) => {
                warn!("Filter application failed: {:?}", _e);
                inc_responses_total();
                let body = single_chunk_response_body("Filter application failed");
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(body)
                    .unwrap())
            }
        }
    } else {
        warn!("No matching route found");
        inc_responses_total();
        let body = single_chunk_response_body("No matching route found");
        Ok(Response::builder().status(StatusCode::NOT_FOUND).body(body).unwrap())
    }
}

/// Apply filters to the incoming request.
async fn apply_filters(
    filters: &[Filter],
    mut req: Request<Incoming>,
) -> Result<Request<Incoming>, hyper::Error> {
    for filter in filters {
        info!("Applying filter: {:?}", filter);
        let start_time = std::time::Instant::now();
        req = filter.apply(req).await?;
        let duration = start_time.elapsed().as_secs_f64();
        info!("Filter {:?} completed in {:.3} ms", filter, duration * 1000.0);
        record_filter_duration(duration);
    }
    Ok(req)
}

async fn forward_request(
    req: Request<Incoming>,
    destination: &str,
) -> Result<Response<BoxBody>, hyper::Error> {
    let fwd_start_time = std::time::Instant::now();
    let uri = match destination.parse::<Uri>() {
        Ok(u) => u,
        Err(_) => {
            warn!("Invalid URI: {}", destination);
            inc_responses_total();
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: invalid URI"))
                .unwrap());
        }
    };

    let host = match uri.host() {
        Some(h) => h,
        None => {
            warn!("URI has no host: {}", uri);
            inc_responses_total();
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: missing host"))
                .unwrap());
        }
    };

    let port = uri.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);

    let stream = match TcpStream::connect(address).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Connection error: {e}");
            inc_responses_total();
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
            warn!("Handshake error: {e}");
            inc_responses_total();
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: handshake failed"))
                .unwrap());
        }
    };

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            warn!("Connection failed: {:?}", err);
        }
    });

    let response = match sender.send_request(req).await {
        Ok(r) => r,
        Err(e) => {
            warn!("Forward request error: {e}");
            inc_responses_total();
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(single_chunk_response_body("Bad Gateway: request failed"))
                .unwrap());
        }
    };

    inc_responses_total();
    let duration = fwd_start_time.elapsed().as_secs_f64();
    info!("Request forwarded in {:.3} ms", duration * 1000.0);

    Ok(response.map(box_pinned_body))
}
