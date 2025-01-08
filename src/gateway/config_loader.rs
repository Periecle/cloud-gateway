use async_trait::async_trait;
use serde_yaml::from_str;
use std::error::Error;
use tokio::fs;

use crate::gateway::config::{Config, FilterConfig, PredicateConfig};
use crate::gateway::filters::*;
use crate::gateway::predicates::*;
use crate::gateway::route::Route;

#[async_trait]
pub trait ConfigLoader {
    async fn load_config(file_path: &str) -> Result<Vec<Route>, Box<dyn Error + Send + Sync>>;
}

pub struct YamlConfigLoader;

#[async_trait]
impl ConfigLoader for YamlConfigLoader {
    async fn load_config(file_path: &str) -> Result<Vec<Route>, Box<dyn Error + Send + Sync>> {
        // Asynchronously read the entire file into memory
        let contents = fs::read_to_string(file_path).await?;
        // Parse YAML in memory
        let config: Config = from_str(&contents)?;

        let routes = config
            .routes
            .into_iter()
            .map(|route_config| {
                let predicates = route_config
                    .predicates
                    .into_iter()
                    .map(|predicate_config| match predicate_config {
                        PredicateConfig::Path { path } => Predicate::Path(PathPredicate { path }),
                        PredicateConfig::Header { header, value } => {
                            Predicate::Header(HeaderPredicate { header, value })
                        }
                        PredicateConfig::QueryParam { param, value } => {
                            Predicate::QueryParam(QueryParamPredicate::new(param, value))
                        }
                        PredicateConfig::Method { method } => {
                            Predicate::Method(MethodPredicate { method })
                        }
                        PredicateConfig::Cookie { name, value } => {
                            Predicate::Cookie(CookiePredicate::new(name, value))
                        }
                        PredicateConfig::Host { patterns } => {
                            Predicate::Host(HostPredicate { patterns })
                        }
                        PredicateConfig::RemoteAddr { addrs } => {
                            Predicate::RemoteAddr(RemoteAddrPredicate {
                                addrs: addrs
                                    .into_iter()
                                    .map(|addr| addr.parse().unwrap())
                                    .collect(),
                            })
                        }
                        PredicateConfig::XForwardedRemoteAddr { addrs } => {
                            Predicate::XForwardedRemoteAddr(XForwardedRemoteAddrPredicate {
                                addrs: addrs
                                    .into_iter()
                                    .map(|addr| addr.parse().unwrap())
                                    .collect(),
                            })
                        }
                    })
                    .collect();

                let filters = route_config
                    .filters
                    .into_iter()
                    .map(|filter_config| match filter_config {
                        FilterConfig::AddRequestHeader { name, value } => {
                            Filter::AddRequestHeader(AddRequestHeader::new(name, value))
                        }
                        FilterConfig::AddRequestParameter { name, value } => {
                            Filter::AddRequestParameters(AddRequestParameter::new(name, value))
                        }
                        FilterConfig::AddRequestHeadersIfNotPresent { headers } => {
                            Filter::AddRequestHeadersIfNotPresent(
                                AddRequestHeadersIfNotPresent::new(headers),
                            )
                        }
                    })
                    .collect();

                Route {
                    id: route_config.id,
                    predicates,
                    filters,
                    destination: route_config.destination,
                }
            })
            .collect();

        Ok(routes)
    }
}
