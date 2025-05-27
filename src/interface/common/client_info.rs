use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tonic::Request;

#[derive(Debug, Deserialize, Serialize)]
pub struct GeoLocation {
    pub city: String,
    pub country: String,
    pub region: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn get_client_ip<T>(request: &Request<T>) -> Option<String> {
    request
        .remote_addr()
        .map(|addr: SocketAddr| addr.ip().to_string())
}

pub fn get_device_info<T>(request: &Request<T>) -> Option<String> {
    request
        .metadata()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok().map(String::from))
}

pub async fn get_location(ip: &str) -> Option<GeoLocation> {
    let url = format!("https://ipapi.co/{}/json/", ip);

    match reqwest::get(&url).await {
        Ok(response) => match response.json::<GeoLocation>().await {
            Ok(geo_info) => Some(geo_info),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
