use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub routes: Vec<RouteConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteConfig {
    pub id: String,
    pub destination: String,
    pub predicates: Vec<PredicateConfig>,
    pub filters: Option<Vec<FilterConfig>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum PredicateConfig {
    Path { path: String },
    Header { header: String, value: String },
    QueryParam { param: String, value: String },
    Method { method: String },
    Cookie { name: String, value: String },
    Host { patterns: Vec<String> },
    RemoteAddr { addrs: Vec<String> },
    XForwardedRemoteAddr { addrs: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FilterConfig {
    AddRequestHeader { name: String, value: String },
    AddRequestHeadersIfNotPresent { headers: Vec<(String, String)> },
    AddRequestParameter { name: String, value: String },
}
