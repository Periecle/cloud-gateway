use std::error::Error;
use std::fs::File;

use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub routes: Vec<RouteConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteConfig {
    pub id: String,
    pub destination: String,
    pub predicates: Vec<PredicateConfig>,
    pub filters: Vec<FilterConfig>,
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
    // Add other filters here...
}


impl Config {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config_file = File::open(file_path)?;
        let config: Config = from_reader(config_file)?;
        Ok(config)
    }
}
