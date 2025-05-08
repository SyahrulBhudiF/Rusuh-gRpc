use crate::cfg;
use crate::domain::entity::user::User;
use crate::domain::redis_repository::RedisRepository;
use crate::domain::repository::Repository;
use crate::interceptor::auth_interceptor::{extract_token_from_metadata, validate_access_token};
use crate::pb::auth::auth_service_server::AuthService;
use crate::pb::auth::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RegisterRequest, RegisterResponse,
    User as ProtoUser,
};
use crate::util::client_info::{get_client_ip, get_device_info, get_location};
use crate::util::jwt::Token;
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct AuthServiceImpl {
    user_repo: Arc<dyn Repository<User> + Send + Sync>,
    redis_repo: Arc<dyn RedisRepository + Send + Sync>,
}

impl AuthServiceImpl {
    pub fn new(
        user_repo: Arc<dyn Repository<User> + Send + Sync>,
        redis_repo: Arc<dyn RedisRepository + Send + Sync>,
    ) -> Self {
        AuthServiceImpl {
            user_repo,
            redis_repo,
        }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let register_req = request.into_inner();

        let hashed_password = hash(&register_req.password, DEFAULT_COST).map_err(|_| {
            error!("Failed to hash password");
            Status::internal("Failed to hash password")
        })?;

        let user = User::new(register_req.email, hashed_password);

        self.user_repo.save(&user).await.map_err(|e| {
            error!("Failed to save user: {}", e);
            Status::internal(format!("Failed to save user: {}", e))
        })?;

        let proto_user = ProtoUser {
            id: user.id.to_string(),
            email: user.email.clone(),
        };

        let response = RegisterResponse {
            user: Some(proto_user),
        };

        info!("User registered successfully: {}", user.email);
        Ok(Response::new(response))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let login_req = request.into_inner();

        if let Some(user) = self
            .user_repo
            .find_by_coll("email", &login_req.email)
            .await
            .map_err(|_| {
                error!("Failed to query user");
                Status::not_found("Failed to query user")
            })?
        {
            return if verify(&login_req.password, &user.password).unwrap_or(false) {
                let (access_token, refresh_token) = Token::create_tokens(user.id.to_string())
                    .await
                    .map_err(|_| {
                        error!("Failed to generate tokens");
                        Status::internal("Failed to generate tokens")
                    })?;

                let user_json = serde_json::to_string(&user).map_err(|_| {
                    error!("Failed to serialize user");
                    Status::internal("Failed to serialize user")
                })?;

                self.redis_repo
                    .set_value(&access_token, &*user_json)
                    .await
                    .expect("Failed to set value in Redis at Login");
                info!("Redis set value for user: {}", user_json);

                let response = LoginResponse {
                    access_token,
                    refresh_token,
                };

                info!("User logged in successfully: {}", user.email);
                Ok(Response::new(response))
            } else {
                error!("Invalid password or email for user: {}", login_req.email);
                Err(Status::unauthenticated("Invalid password or email"))
            };
        }

        error!("Invalid email or password for user: {}", login_req.email);
        Err(Status::unauthenticated("Invalid email or password"))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let metadata = request.metadata().clone();
        validate_access_token(&metadata, &self.redis_repo).await?;

        let access_token = extract_token_from_metadata(&metadata)
            .map_err(|e| {
                error!("Failed to extract token: {}", e);
                Status::unauthenticated("Invalid token")
            })?
            .to_string();

        let logout_req = request.into_inner();
        let refresh_token = &logout_req.refresh_token;

        self.redis_repo
            .ensure_not_blacklisted(refresh_token)
            .await?;

        let config = cfg();
        Token::validate_token(refresh_token, &config.refresh_secret).map_err(|e| {
            error!("Invalid refresh token: {}", e);
            Status::unauthenticated("Invalid refresh token")
        })?;

        self.redis_repo
            .blacklist_token(refresh_token)
            .await
            .map_err(|e| {
                error!("Failed to blacklist refresh token: {}", e);
                Status::internal("Logout failed")
            })?;

        self.redis_repo.blacklist_token(&access_token).await.ok();

        info!("Logout success: {}", refresh_token);

        Ok(Response::new(LogoutResponse {
            message: "Logout successful".to_string(),
        }))
    }
}
