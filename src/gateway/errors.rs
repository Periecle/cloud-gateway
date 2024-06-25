use std::error::Error as StdError;
use std::fmt;
use std::fmt::write;

use hyper::Error as HyperError;
use hyper::http::uri::InvalidUri;
use tokio::io::Error as IoError;

#[derive(Debug)]
pub enum GatewayError {
    UriParseError,
    NoHostError,
    HyperError(HyperError),
    IoError(IoError),
    NoRouteMatched,
    ConnectionFailed(Box<dyn StdError + Send + Sync>)
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::UriParseError => write!(f, "Failed to parse URI"),
            GatewayError::NoHostError => write!(f, "URI has no host"),
            GatewayError::HyperError(err) => write!(f, "Hyper error: {}", err),
            GatewayError::IoError(err) => write!(f, "I/O error: {}", err),
            GatewayError::NoRouteMatched => write!(f, "No route matched"),
            GatewayError::ConnectionFailed(err) => write!(f, "Connection error: {}", err)
        }
    }
}

impl StdError for GatewayError {}

impl From<InvalidUri> for GatewayError {
    fn from(_: InvalidUri) -> Self {
        GatewayError::UriParseError
    }
}

impl From<HyperError> for GatewayError {
    fn from(err: HyperError) -> Self {
        GatewayError::HyperError(err)
    }
}

impl From<IoError> for GatewayError {
    fn from(err: IoError) -> Self {
        GatewayError::IoError(err)
    }
}