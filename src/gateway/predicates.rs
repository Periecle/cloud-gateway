pub mod path;
pub mod header;
pub mod query_param;
pub mod method;
pub mod after;
pub mod before;
pub mod between;
pub mod cookie;
pub mod host;
pub mod remote_addr;
pub mod x_forwarded_remote_addr;

pub trait Predicate {
    fn evaluate(&self, request: &crate::gateway::request::Request) -> bool;
}

pub use path::PathPredicate;
pub use header::HeaderPredicate;
pub use query_param::QueryParamPredicate;
pub use method::MethodPredicate;
pub use after::AfterPredicate;
pub use before::BeforePredicate;
pub use between::BetweenPredicate;
pub use cookie::CookiePredicate;
pub use host::HostPredicate;
pub use remote_addr::RemoteAddrPredicate;
pub use x_forwarded_remote_addr::XForwardedRemoteAddrPredicate;
