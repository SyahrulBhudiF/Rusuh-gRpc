use crate::config::db::get_db_pool;
use crate::infrastructure::db::user_adapter::UserAdapter;
use crate::infrastructure::db::user_session_adapter::UserSessionAdapter;
use crate::infrastructure::redis::redis_adapter::RedisAdapter;
use crate::interface::grpc::handler::auth_handler::AuthHandler;
use crate::interface::grpc::layer::logging_layer::LoggingLayer;
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
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../../descriptor.bin");

pub async fn server() -> Result<(), Box<dyn error::Error>> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
        .init();

    let pool = get_db_pool().await?;

    let user_repo = Arc::new(UserAdapter::new(pool.clone()));
    let redis_repo = Arc::new(RedisAdapter::new());
    let session_repo = Arc::new(UserSessionAdapter::new(pool.clone()));

    let auth_handler = AuthHandler::new(user_repo, session_repo, redis_repo);

    let addr = "0.0.0.0:50051".parse()?;
    info!("Server listening on {}", addr);

    let middleware_stack = ServiceBuilder::new()
        .layer(LoggingLayer)
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
