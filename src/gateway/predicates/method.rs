use hyper::Request;

use super::Evaluable;

#[derive(Clone, Debug)]
pub struct MethodPredicate {
    pub method: String,
}

impl<T> Evaluable<T> for MethodPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        request.method().as_str() == self.method
    }
}
