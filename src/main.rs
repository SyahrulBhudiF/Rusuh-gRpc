use rusuh_grpc::core::server::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server().await
}
