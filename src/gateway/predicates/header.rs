use regex::Regex;

use hyper::Request;

use super::Evaluable;

#[derive(Clone, Debug)]
pub struct HeaderPredicate {
    pub header: String,
    pub value: String,
}

impl <T> Evaluable<T> for HeaderPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        let re = Regex::new(&self.value).unwrap();
        request.headers().get(&self.header).map_or(false, |v| re.is_match(v.to_str().unwrap()))
    }
}
