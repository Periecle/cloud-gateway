use async_trait::async_trait;
use hyper::body::Incoming;

pub use add_request_header::AddRequestHeader;
pub use add_request_headers_if_not_present::AddRequestHeadersIfNotPresent;
pub use add_request_parameter::AddRequestParameter;

use crate::gateway::Request;

pub mod add_request_header;
pub mod add_request_headers_if_not_present;
pub mod add_request_parameter;

pub type FilteredResult = Result<Request<Incoming>, hyper::Error>;

#[async_trait]
pub trait Filterable: Send + Sync {
    async fn apply(&self, req: Request<Incoming>) -> FilteredResult;
}

#[derive(Clone, Debug)]
pub enum Filter {
    AddRequestHeader(AddRequestHeader),
    AddRequestHeadersIfNotPresent(AddRequestHeadersIfNotPresent),
    AddRequestParameters(AddRequestParameter)
    // Add other filter variants here...
}

#[async_trait]
impl Filterable for Filter {
    async fn apply(&self,  req: Request<Incoming>) -> FilteredResult {
        match self {
            Filter::AddRequestHeader(f) => f.apply(req).await,
            Filter::AddRequestHeadersIfNotPresent(f) => f.apply(req).await,
            Filter::AddRequestParameters(f) => f.apply(req).await
            // Match other filter variants here...
        }
    }
}

