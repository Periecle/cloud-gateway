use std::collections::HashMap;
use regex::Regex;

use hyper::Request;

use super::Evaluable;

#[derive(Clone, Debug)]
pub struct QueryParamPredicate {
    pub param: String,
    pub value: Regex,
}

impl QueryParamPredicate {
    pub fn new(param: String, value: String) -> Self {
        Self {
            param,
            value: Regex::new(&value).unwrap(), // Compile regex once
        }
    }
}

impl<T> Evaluable<T> for QueryParamPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        if let Some(query_str) = request.uri().query() {
            let query_params = parse_query_params(query_str);
            if let Some(param_value) = query_params.get(self.param.as_str()) {
                return self.value.is_match(param_value);
            }
        }
        false
    }
}

fn parse_query_params(query: &str) -> HashMap<&str, &str> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut iter = pair.split('=');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                Some((key.trim(), value.trim()))
            } else {
                None
            }
        })
        .collect()
}