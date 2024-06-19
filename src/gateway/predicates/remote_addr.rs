use std::net::{IpAddr, SocketAddr};

use hyper::Request;

use super::Evaluable;

#[derive(Clone, Debug)]
pub struct RemoteAddrPredicate {
    pub addrs: Vec<IpAddr>,
}

impl <T> Evaluable<T> for RemoteAddrPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        let written_value = &request.extensions().get::<SocketAddr>().unwrap().ip();
        self.addrs.contains(written_value)
    }
}
