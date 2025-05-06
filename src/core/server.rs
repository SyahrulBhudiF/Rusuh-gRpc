use crate::config::db::get_db_pool;
use crate::handler::auth_handler::AuthHandler;
use crate::infrastructure::redis_repository::RedisRepositoryImpl;
use crate::infrastructure::user_repository::UserRepositoryImpl;
use crate::pb::auth::auth_service_server::AuthServiceServer;
use std::error;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tracing::info;

const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../../descriptor.bin");

pub async fn server() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let pool = get_db_pool().await?;

    let user_repo = Arc::new(UserRepositoryImpl { pool });
    let redis_repo = Arc::new(RedisRepositoryImpl::new());

    let auth_handler = AuthHandler::new(user_repo, redis_repo);

    let addr = "0.0.0.0:50051".parse()?;
    info!("Server listening on {}", addr);

    let middleware_stack = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(ConcurrencyLimitLayer::new(64))
        .into_inner();

    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    Server::builder()
        .layer(middleware_stack)
        .add_service(AuthServiceServer::new(auth_handler))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
