pub mod predicates;
pub mod filters;
pub mod route;
pub mod config;

use predicates::Predicate;
use hyper::Request;
use route::Route;

pub fn find_matching_route<'a, T>(routes: &'a[Route], request: &Request<T>) -> Option<&'a Route> {
    routes.iter().find(|route| route.matches(request))
}