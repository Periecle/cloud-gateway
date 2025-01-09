use async_trait::async_trait;
use backoff::exponential::ExponentialBackoff;
use backoff::future::retry;
use dashmap::DashMap;
use parking_lot::RwLock;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::gateway::config::Config;
use crate::gateway::config_loader::ConfigLoader;
use crate::gateway::route::Route;
use crate::gateway::spring_config_loader::PropertySource;

#[derive(Debug, Clone)]
struct CacheEntry {
    routes: Vec<Route>,
    timestamp: Instant,
}

#[derive(Debug)]
pub struct SpringConfigLoader {
    config_server_url: String,
    application_name: String,
    profile: String,
    label: Option<String>,
    client: Client,
    cache: Arc<DashMap<String, CacheEntry>>,
    cache_duration: Duration,
}

#[derive(Deserialize, Debug)]
struct SpringConfigResponse {
    name: String,
    profiles: Vec<String>,
    label: Option<String>,
    propertySources: Vec<PropertySource>,
}

#[derive(Deserialize, Debug)]
struct PropertySource {
    name: String,
    source: std::collections::HashMap<String, serde_json::Value>,
}

impl SpringConfigLoader {
    pub fn new(
        config_server_url: String,
        application_name: String,
        profile: String,
        label: Option<String>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config_server_url,
            application_name,
            profile,
            label,
            client,
            cache: Arc::new(DashMap::new()),
            cache_duration: Duration::from_secs(300), // 5 minutes cache duration
        }
    }

    fn build_config_url(&self) -> String {
        match &self.label {
            Some(label) => format!(
                "{}/{}/{}/{}",
                self.config_server_url, self.application_name, self.profile, label
            ),
            None => format!(
                "{}/{}/{}",
                self.config_server_url, self.application_name, self.profile
            ),
        }
    }

    fn get_cache_key(&self) -> String {
        match &self.label {
            Some(label) => format!("{}:{}:{}", self.application_name, self.profile, label),
            None => format!("{}:{}", self.application_name, self.profile),
        }
    }

    async fn fetch_with_retry(&self) -> Result<SpringConfigResponse, Box<dyn Error + Send + Sync>> {
        let url = self.build_config_url();
        let client = self.client.clone();

        let operation = || async {
            let response = client
                .get(&url)
                .header("Accept", "application/json")
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => Ok(response.json::<SpringConfigResponse>().await?),
                status if status.is_server_error() => {
                    Err(backoff::Error::transient(format!("Server error: {}", status)))
                }
                status => Err(backoff::Error::permanent(format!(
                    "Unexpected status: {}",
                    status
                ))),
            }
        };

        let backoff = ExponentialBackoff {
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(10),
            max_elapsed_time: Some(Duration::from_secs(30)),
            ..ExponentialBackoff::default()
        };

        Ok(retry(backoff, operation).await?)
    }

    pub async fn load(&self) -> Result<Vec<Route>, Box<dyn Error + Send + Sync>> {
        let cache_key = self.get_cache_key();

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.timestamp.elapsed() < self.cache_duration {
                return Ok(cached.routes.clone());
            }
        }

        // Fetch from config server with retry
        let config_response = self.fetch_with_retry().await?;

        // Find the first property source that contains our routes configuration
        let routes_config = config_response
            .propertySources
            .iter()
            .find_map(|ps| ps.source.get("cloud.gateway.routes"))
            .ok_or("No routes configuration found")?;

        // Parse the routes configuration
        let config: Config = serde_json::from_value(routes_config.clone())?;

        // Convert config to routes
        let routes = config
            .routes
            .into_iter()
            .map(|route_config| Route {
                id: route_config.id,
                predicates: route_config
                    .predicates
                    .into_iter()
                    .map(|p| match p {
                        // Copy your existing predicate conversion logic here
                        // ...
                    })
                    .collect(),
                filters: route_config
                    .filters
                    .into_iter()
                    .map(|f| match f {
                        // Copy your existing filter conversion logic here
                        // ...
                    })
                    .collect(),
                destination: route_config.destination,
            })
            .collect();

        // Update cache
        self.cache.insert(
            cache_key,
            CacheEntry {
                routes: routes.clone(),
                timestamp: Instant::now(),
            },
        );

        Ok(routes)
    }

    // Method to clear the cache if needed
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    // Method to set custom cache duration
    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}

#[async_trait]
impl ConfigLoader for SpringConfigLoader {
    async fn load_config(file_path: &str) -> Result<Vec<Route>, Box<dyn Error + Send + Sync>> {
        unimplemented!("Use SpringConfigLoader::new() instead");
    }
}
