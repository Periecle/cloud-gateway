use crate::gateway::predicates::Evaluable;
use hyper::Request;

#[derive(Clone, Debug)]
pub struct PathPredicate {
    pub path: String,
}

impl <T> Evaluable<T> for PathPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        &request.uri().path() == &self.path
    }
}