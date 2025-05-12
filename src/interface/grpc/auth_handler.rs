use crate::application::auth_use_case::AuthUseCase;
use crate::domain::entity::user::User;
use crate::domain::port::db_port::DbPort;
use crate::domain::port::redis_port::RedisPort;
use crate::pb::auth::auth_handler_server::AuthHandler as Handler;
use crate::pb::auth::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RegisterRequest, RegisterResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AuthHandler {
    auth_service: AuthUseCase,
}

impl AuthHandler {
    pub fn new(
        port: Arc<dyn DbPort<User> + Send + Sync>,
        redis_port: Arc<dyn RedisPort + Send + Sync>,
    ) -> Self {
        AuthHandler {
            auth_service: AuthUseCase::new(port, redis_port),
        }
    }
}

#[tonic::async_trait]
impl Handler for AuthHandler {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        self.auth_service.register(request).await
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        self.auth_service.login(request).await
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        self.auth_service.logout(request).await
    }
}
