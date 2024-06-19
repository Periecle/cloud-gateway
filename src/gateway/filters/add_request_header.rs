use async_trait::async_trait;
use http::{HeaderName, HeaderValue};
use hyper::{body::Incoming, Request};

use super::Filterable;
use super::FilteredResult;

#[derive(Clone, Debug)]
pub struct AddRequestHeader {
    pub name: String,
    pub value: String,
}

impl AddRequestHeader {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[async_trait]
impl Filterable for AddRequestHeader {
    async fn apply(&self, mut req: Request<Incoming>) -> FilteredResult {
        req.headers_mut().insert(
            self.name.parse::<HeaderName>().unwrap(),
            self.value.parse::<HeaderValue>().unwrap(),
        );
        Ok(req)
    }
}