use std::net::SocketAddr;
use std::sync::Arc;

use log::{error, info};

use gateway::config_loader::{self, YamlConfigLoader};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use gateway::route::Route;
mod gateway;
use config_loader::ConfigLoader;
use responder::responder;

use gateway::eureka;
mod gateway_metrics;
mod responder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    if let Err(e) = eureka::register_in_eureka().await {
        eprintln!("Failed to register in Eureka: {}", e);
    }

    tokio::spawn(eureka::start_heartbeat());

    info!("Initializing cloud gateway...");
    let routes: Vec<Route> = YamlConfigLoader::load_config("config.yaml").await?;
    info!("Configuration loaded");
    let routes = Arc::new(RwLock::new(routes));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let routes_clone = routes.clone();
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| responder(req, routes_clone.clone(), remote_addr)),
                )
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
