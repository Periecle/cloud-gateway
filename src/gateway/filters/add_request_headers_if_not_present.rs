use async_trait::async_trait;
use http::{HeaderName, HeaderValue};
use hyper::{body::Incoming, Request};

use super::Filterable;
use super::FilteredResult;

#[derive(Clone, Debug)]
pub struct AddRequestHeadersIfNotPresent {
    pub headers: Vec<(String, String)>,
}

impl AddRequestHeadersIfNotPresent {
    pub fn new(headers: Vec<(String, String)>) -> Self {
        Self { headers }
    }
}

#[async_trait]
impl Filterable for AddRequestHeadersIfNotPresent {
    async fn apply(&self, mut req: Request<Incoming>) -> FilteredResult {
        for (name, value) in &self.headers {
            let header_name = name.parse::<HeaderName>().unwrap();
            if !req.headers().contains_key(&header_name) {
                req.headers_mut().insert(
                    header_name,
                    value.parse::<HeaderValue>().unwrap(),
                );
            }
        }
        Ok(req)
    }
}
