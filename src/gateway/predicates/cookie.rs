use std::collections::HashMap;
use regex::Regex;

use super::Evaluable;
use hyper::Request;

#[derive(Clone, Debug)]
pub struct CookiePredicate {
    pub name: String,
    pub value: Regex, // Precompiled regex
}

impl CookiePredicate {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value: Regex::new(&value).unwrap(), // Compile regex once
        }
    }
}

impl <T> Evaluable<T> for CookiePredicate {

    fn evaluate(&self, request: &Request<T>) -> bool {
        if let Some(cookie_header) = request.headers().get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                let cookies = parse_cookies(cookie_str);
                if let Some(cookie_value) = cookies.get(self.name.as_str()) {
                    return self.value.is_match(cookie_value);
                }
            }
        }
        false
    }
}

fn parse_cookies(cookie_str: &str) -> HashMap<&str, &str> {
    cookie_str
        .split(';')
        .filter_map(|c| {
            let mut iter = c.split('=');
            if let (Some(name), Some(value)) = (iter.next(), iter.next()) {
                Some((name.trim(), value.trim()))
            } else {
                None
            }
        })
        .collect()
}
