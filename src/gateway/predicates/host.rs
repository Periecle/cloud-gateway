use regex::Regex;

use hyper::Request;

use super::Evaluable;

#[derive(Clone, Debug)]
pub struct HostPredicate {
    pub patterns: Vec<String>,
}

impl<T> Evaluable<T> for HostPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        self.patterns.iter().any(|pattern| {
            let re = Regex::new(&pattern.replace("**", ".*")).unwrap();
            re.is_match(request.uri().host().unwrap())
        })
    }
}
