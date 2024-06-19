use async_trait::async_trait;
use http::uri::Uri;
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
            query_pairs.extend_pairs(query.split('&').map(|s| {
                let mut split = s.splitn(2, '=');
                (split.next().unwrap_or_default(), split.next().unwrap_or_default())
            }));
        }

        query_pairs.append_pair(&self.name, &self.value);
        let query = query_pairs.finish();
        uri_parts.path_and_query = Some(http::uri::PathAndQuery::from_static(query.as_str()));
        let new_uri = Uri::from_parts(uri_parts).unwrap();
        *req.uri_mut() = new_uri;

        Ok(req)
    }
}
