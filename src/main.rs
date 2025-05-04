use rusuh_grpc::config::db::get_db_pool;
use rusuh_grpc::handler::auth_handler::AuthHandler;
use rusuh_grpc::infrastructure::user_repository::UserRepositoryImpl;
use rusuh_grpc::pb::auth::auth_service_server::AuthServiceServer;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;

const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../descriptor.bin");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let pool = get_db_pool().await?;
    let user_repo = Box::new(UserRepositoryImpl { pool });
    let auth_handler = AuthHandler::new(user_repo);

    let addr = "0.0.0.0:50051".parse()?;
    println!("Server listening on {}", addr);

    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    Server::builder()
        .add_service(AuthServiceServer::new(auth_handler))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
