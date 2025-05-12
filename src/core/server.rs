use crate::config::db::get_db_pool;
use crate::infrastructure::db::user_adapter::UserAdapter;
use crate::infrastructure::redis::redis_adapter::RedisAdapter;
use crate::interface::grpc::auth_handler::AuthHandler;
use crate::pb::auth::auth_handler_server::AuthHandlerServer;
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

    let user_repo = Arc::new(UserAdapter { pool });
    let redis_repo = Arc::new(RedisAdapter::new());

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
        .add_service(AuthHandlerServer::new(auth_handler))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
