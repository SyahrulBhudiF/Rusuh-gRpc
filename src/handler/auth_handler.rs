use crate::domain::entity::user::User;
use crate::domain::redis_repository::RedisRepository;
use crate::domain::repository::Repository;
use crate::pb::auth::auth_service_server::AuthService;
use crate::pb::auth::{LoginRequest, RegisterRequest};
use crate::pb::auth::{LoginResponse, RegisterResponse};
use crate::service::auth_service::AuthServiceImpl;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AuthHandler {
    auth_service: AuthServiceImpl,
}

impl AuthHandler {
    pub fn new(
        user_repo: Arc<dyn Repository<User> + Send + Sync>,
        redis_repo: Arc<dyn RedisRepository + Send + Sync>,
    ) -> Self {
        AuthHandler {
            auth_service: AuthServiceImpl::new(user_repo, redis_repo),
        }
    }
}

#[tonic::async_trait]
impl AuthService for AuthHandler {
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
}
