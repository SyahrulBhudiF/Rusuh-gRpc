use crate::domain::entity::user::User;
use crate::domain::jwt::Token;
use crate::domain::redis_repository::RedisRepository;
use crate::domain::repository::Repository;
use crate::interceptor::auth_interceptor::{extract_token_from_metadata, validate_access_token};
use crate::pb::auth::auth_service_server::AuthService;
use crate::pb::auth::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RegisterRequest, RegisterResponse,
    User as ProtoUser,
};
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
        validate_access_token(&metadata)?;

        let access_token = match extract_token_from_metadata(&metadata) {
            Ok(token) => token.to_string(),
            Err(e) => {
                error!("Failed to extract token from metadata: {}", e);
                return Err(Status::unauthenticated("Invalid token"));
            }
        };

        let logout_req = request.into_inner();

        match self.redis_repo.get_value(&access_token).await {
            Ok(value) if value == "BLACKLISTED" => {
                error!("Access token already blacklisted, possible reuse attempt");
                return Err(Status::unauthenticated(
                    "Token already invalidated or expired",
                ));
            }
            _ => {}
        }

        match self.redis_repo.get_value(&logout_req.refresh_token).await {
            Ok(value) if value == "BLACKLISTED" => {
                error!("Refresh token already blacklisted, possible reuse attempt");
                return Err(Status::unauthenticated("Refresh token already invalidated"));
            }
            _ => {}
        }

        match Token::validate_token(&logout_req.refresh_token, "REFRESH_SECRET") {
            Ok(_) => {
                if let Err(e) = self
                    .redis_repo
                    .set_value(&logout_req.refresh_token, "BLACKLISTED")
                    .await
                {
                    error!("Failed to blacklist refresh token: {}", e.to_string());
                    return Err(Status::internal("Failed to complete logout process"));
                }

                if let Err(e) = self
                    .redis_repo
                    .set_value(&access_token, "BLACKLISTED")
                    .await
                {
                    error!("Failed to blacklist access token: {}", e.to_string());
                }

                info!("User logged out successfully: {}", logout_req.refresh_token);

                let response = LogoutResponse {
                    message: "Logout successful".to_string(),
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to validate refresh token: {}", e);
                Err(Status::unauthenticated("Invalid refresh token"))
            }
        }
    }
}
