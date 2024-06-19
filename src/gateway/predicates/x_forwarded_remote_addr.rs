use std::net::IpAddr;

use hyper::Request;

use super::Evaluable;

#[derive(Clone, Debug)]
pub struct XForwardedRemoteAddrPredicate {
    pub addrs: Vec<IpAddr>,
}

impl<T> Evaluable<T> for XForwardedRemoteAddrPredicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        if let Some(header_value) = request.headers().get("x-forwarded-for") {
            if let Ok(header_str) = header_value.to_str() {
                return header_str
                    .split(',')
                    .map(|ip| ip.trim().parse::<IpAddr>())
                    .filter_map(Result::ok)
                    .any(|addr| self.addrs.contains(&addr));
            }
        }
        false
    }
}