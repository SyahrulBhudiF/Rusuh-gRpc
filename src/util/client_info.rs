use serde::Deserialize;
use std::net::SocketAddr;
use tonic::{Request, Status};

#[derive(Debug, Deserialize)]
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

pub async fn get_location(ip: &str) -> Result<GeoLocation, Status> {
    let url = format!("https://ipapi.co/{}/json/", ip);

    match reqwest::get(&url).await {
        Ok(response) => match response.json::<GeoLocation>().await {
            Ok(geo_info) => Ok(geo_info),
            Err(_) => Err(Status::internal("Error when Accessing geolocation service")),
        },
        Err(_) => Err(Status::internal("Error when Accessing geolocation service")),
    }
}
