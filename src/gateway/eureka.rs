use std::time::Duration;

use reqwest::Client;
use serde::Serialize;
use tokio::time;

#[derive(Serialize)]
struct InstanceInfo {
    instance: Instance,
}

#[derive(Serialize)]
struct Instance {
    instanceId: String,
    hostName: String,
    app: String,
    ipAddr: String,
    status: String,
    port: Port,
    vipAddress: String,
    dataCenterInfo: DataCenterInfo,
}

#[derive(Serialize)]
struct Port {
    #[serde(rename = "$")]
    value: u16,
    #[serde(rename = "@enabled")]
    enabled: bool,
}

#[derive(Serialize)]
struct DataCenterInfo {
    name: String,
    #[serde(rename = "@class")]
    class: String,
}

pub async fn register_in_eureka() -> Result<(), reqwest::Error> {
    let eureka_url = "http://localhost:8761/eureka/apps/gateway";

    let instance_info = InstanceInfo {
        instance: Instance {
            instanceId: "rust-app-2".to_string(),
            hostName: "localhost".to_string(),
            app: "gateway".to_string(),
            ipAddr: "127.0.0.1".to_string(),
            status: "UP".to_string(),
            port: Port { value: 8080, enabled: true },
            vipAddress: "gateway".to_string(),
            dataCenterInfo: DataCenterInfo {
                name: "MyOwn".to_string(),
                class: "com.netflix.appinfo.InstanceInfo$DefaultDataCenterInfo".to_string(),
            },
        },
    };

    let client = Client::new();

    let response = client.post(eureka_url).json(&instance_info).send().await?;

    if response.status().is_success() {
        println!("Successfully registered with Eureka");
    } else {
        println!("Failed to register with Eureka: {:?}", response.text().await?);
    }

    Ok(())
}

pub async fn start_heartbeat() {
    let mut interval = time::interval(Duration::from_secs(30)); // Send every 30 seconds
    loop {
        interval.tick().await;
        if let Err(e) = send_heartbeat().await {
            eprintln!("Error sending heartbeat: {}", e);
        }
    }
}

async fn send_heartbeat() -> Result<(), reqwest::Error> {
    let eureka_url = "http://localhost:8761/eureka/apps/gateway/rust-app-2";

    let client = Client::new();
    let response = client.put(eureka_url).send().await?;

    if response.status().is_success() {
        println!("Heartbeat sent successfully");
    } else {
        println!("Failed to send heartbeat: {:?}", response.text().await?);
    }

    Ok(())
}
