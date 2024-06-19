use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http::Uri;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response};
use hyper::body::{Body, Buf, Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use gateway::config::Config;
use gateway::config::FilterConfig;
use gateway::config::PredicateConfig;
use gateway::filters;
use gateway::filters::Filter;
use gateway::filters::Filterable;
use gateway::find_matching_route;
use gateway::predicates;
use gateway::predicates::Predicate;
use gateway::route::Route;

mod gateway;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_file("config.yaml")?;

    let routes: Vec<Route> = config
        .routes
        .into_iter()
        .map(|route_config| {
            let predicates = route_config
                .predicates
                .into_iter()
                .map(|predicate_config| match predicate_config {
                    PredicateConfig::Path { path } => {
                        Predicate::Path(predicates::PathPredicate { path })
                    }
                    PredicateConfig::Header { header, value } => {
                        Predicate::Header(predicates::HeaderPredicate { header, value })
                    }
                    PredicateConfig::QueryParam { param, value } => {
                        Predicate::QueryParam(predicates::QueryParamPredicate::new(param, value))
                    }
                    PredicateConfig::Method { method } => {
                        Predicate::Method(predicates::MethodPredicate { method })
                    }
                    PredicateConfig::Cookie { name, value } => {
                        Predicate::Cookie(predicates::CookiePredicate::new(name, value))
                    }
                    PredicateConfig::Host { patterns } => {
                        Predicate::Host(predicates::HostPredicate { patterns })
                    }
                    PredicateConfig::RemoteAddr { addrs } => {
                        Predicate::RemoteAddr(predicates::RemoteAddrPredicate {
                            addrs: addrs.into_iter().map(|addr| addr.parse().unwrap()).collect(),
                        })
                    }
                    PredicateConfig::XForwardedRemoteAddr { addrs } => {
                        Predicate::XForwardedRemoteAddr(predicates::XForwardedRemoteAddrPredicate {
                            addrs: addrs.into_iter().map(|addr| addr.parse().unwrap()).collect(),
                        })
                    }
                })
                .collect();

            let filters = route_config
                .filters
                .into_iter()
                .map(|filter_config| {
                    match filter_config {
                        FilterConfig::AddRequestHeader { name, value } => {
                            Filter::AddRequestHeader(filters::AddRequestHeader::new(name, value))
                        } // Add other filters here...
                    }
                })
                .collect();

            Route {
                id: route_config.id,
                predicates,
                filters,
                destination: route_config.destination,
            }
        })
        .collect();

    let routes = Arc::new(RwLock::new(routes));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, remote_addr) = listener.accept().await?;

        // Clone the Arc for routes to pass into the task
        let routes = routes.clone();

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `responder` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(move |req| responder(req, routes.clone(), remote_addr)))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn responder(
    mut req: Request<hyper::body::Incoming>,
    routes: Arc<RwLock<Vec<Route>>>,
    remote_addr: SocketAddr,
) -> Result<Response<Box<dyn Body<Data=Bytes, Error=(hyper::Error)>>>, hyper::Error> {
    // Insert the remote address into the request extensions
    req.extensions_mut().insert(remote_addr);

    // Read the routes from the shared state
    let routes = routes.read().await;

    if let Some(route) = find_matching_route(&routes, &req) {
        // Apply filters to the request
        match apply_filters(&route.filters, req).await {
            Ok((filtered_req)) => {
                // Forward the filtered request to the route's destination
                forward_request(filtered_req, &route.destination.clone()).await.as
            }
            Err(_) => Ok(Response::new(Box::new(Full::<Bytes>::from("Hello World"))))
        }
    } else {
        Ok(Response::builder().status(404).body(Box::new(Bytes::from("No matching route found")).unwrap()))
    }
}

/// Applies the given filters to the request.
async fn apply_filters(
    filters: &[Filter],
    mut req: Request<hyper::body::Incoming>,
) -> Result<Request<Incoming>, hyper::Error> {
    // Apply each filter to the request and response
    for filter in filters {
     req = filter.apply(req).await?;
    }

    Ok(req)
}

async fn mock_forward_request(
    _: Request<hyper::body::Incoming>,
    destination: &str,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // Forwarding logic here...
    Ok(Response::new(Full::new(Bytes::from(format!("Forwarded to {}", destination)))))
}

async fn forward_request(
    req: Request<hyper::body::Incoming>,
    destination: &str,
) -> Result<Response<Box<hyper::body::Incoming>>, hyper::Error> {
    // Parse our URL...
    let url = destination.parse::<Uri>().unwrap();

    // Get the host and the port
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{}:{}", host, port);

    // Open a TCP connection to the remote host
    let stream = TcpStream::connect(address).await.unwrap();

    // Use an adapter to access something implementing `tokio::io` traits as if they implement
    // `hyper::rt` IO traits.
    let io = TokioIo::new(stream);

    // Create the Hyper client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await{
            println!("Connection failed: {:?}", err);
        }
    });

    let res = sender.send_request(req).await.unwrap().boxed()
        .collect();

    println!("Response status: {}", res.status());

    Ok(res)
}