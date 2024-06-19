use crate::gateway::filters::Filter;
use crate::gateway::Predicate;
use crate::gateway::predicates::Evaluable;

use hyper::Request;

#[derive(Clone, Debug)]
pub struct Route {
    pub id: String,
    pub predicates: Vec<Predicate>,
    pub filters: Vec<Filter>,
    pub destination: String,
}

impl Route {
    pub fn matches<T>(&self, request: &Request<T>) -> bool {
        self.predicates.iter().all(|predicate| predicate.evaluate(request))
    }
}