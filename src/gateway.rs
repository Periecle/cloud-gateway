mod gateway {

    pub use crate::predicates::Predicate;
    pub use crate::request::Request;
    pub use crate::route::Route;

    pub fn find_matching_route<'a, P: Predicate>(
        routes: &'a [Route<P>],
        request: &Request,
    ) -> Option<&'a Route<P>> {
        routes.iter().find(|route| route.matches(request))
    }
}
