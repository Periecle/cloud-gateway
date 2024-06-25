use std::str::FromStr;
use async_trait::async_trait;
use http::Uri;
use http::uri::PathAndQuery;
use hyper::{body::Incoming, Request};

use super::Filterable;
use super::FilteredResult;

#[derive(Clone, Debug)]
pub struct AddRequestParameter {
    pub name: String,
    pub value: String,
}

impl AddRequestParameter {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[async_trait]
impl Filterable for AddRequestParameter {
    async fn apply(&self, mut req: Request<Incoming>) -> FilteredResult {
        let mut uri_parts = req.uri().clone().into_parts();
        let mut query_pairs = form_urlencoded::Serializer::new(String::new());

        if let Some(query) = req.uri().query() {
            query_pairs.extend_pairs(form_urlencoded::parse(query.as_bytes()));
        }

        query_pairs.append_pair(&self.name, &self.value);
        let new_query = query_pairs.finish();
        let path = uri_parts.path_and_query.as_ref().map_or("/", |pq| pq.path());
        uri_parts.path_and_query = Some(PathAndQuery::from_str(&format!("{}?{}", path, new_query)).unwrap());

        let new_uri = Uri::from_parts(uri_parts).unwrap();
        *req.uri_mut() = new_uri;

        Ok(req)
    }
}