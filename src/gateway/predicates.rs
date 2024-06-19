pub mod path;
pub mod header;
pub mod query_param;
pub mod method;
pub mod cookie;
pub mod host;
pub mod remote_addr;
pub mod x_forwarded_remote_addr;

use std::fmt::Debug;
use hyper::Request;

pub trait Evaluable<T>: Debug + Send + Sync {
    fn evaluate(&self, request: &Request<T>) -> bool;
}

#[derive(Clone, Debug)]
pub enum Predicate {
    Path(PathPredicate),
    Header(HeaderPredicate),
    QueryParam(QueryParamPredicate),
    Method(MethodPredicate),
    Cookie(CookiePredicate),
    Host(HostPredicate),
    RemoteAddr(RemoteAddrPredicate),
    XForwardedRemoteAddr(XForwardedRemoteAddrPredicate),
}

impl <T> Evaluable<T> for Predicate {
    fn evaluate(&self, request: &Request<T>) -> bool {
        match self {
            Predicate::Path(p) => p.evaluate(request),
            Predicate::Header(p) => p.evaluate(request),
            Predicate::QueryParam(p) => p.evaluate(request),
            Predicate::Method(p) => p.evaluate(request),
            Predicate::Cookie(p) => p.evaluate(request),
            Predicate::Host(p) => p.evaluate(request),
            Predicate::RemoteAddr(p) => p.evaluate(request),
            Predicate::XForwardedRemoteAddr(p) => p.evaluate(request),
        }
    }
}

pub use path::PathPredicate;
pub use header::HeaderPredicate;
pub use query_param::QueryParamPredicate;
pub use method::MethodPredicate;
pub use cookie::CookiePredicate;
pub use host::HostPredicate;
pub use remote_addr::RemoteAddrPredicate;
pub use x_forwarded_remote_addr::XForwardedRemoteAddrPredicate;
