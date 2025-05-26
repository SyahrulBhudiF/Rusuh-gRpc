use crate::cfg;
use crate::domain::entity::user::{User, UserStatus};
use crate::domain::entity::user_sessions::UserSessions;
use crate::domain::port::db_port::DbPort;
use crate::domain::port::redis_port::RedisPort;
use crate::domain::service::jwt_service::Token;
use crate::interface::common::client_info::{
    GeoLocation, get_client_ip, get_device_info, get_location,
};
use crate::interface::interceptor::auth_interceptor::{
    extract_token_from_metadata, validate_access_token,
};
use crate::pb::auth::auth_handler_server::AuthHandler;
use crate::pb::auth::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, RegisterRequest, RegisterResponse,
    User as ProtoUser,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct AuthUseCase {
    adapter: Arc<dyn DbPort<User> + Send + Sync>,
    session: Arc<dyn DbPort<UserSessions> + Send + Sync>,
    redis_adapter: Arc<dyn RedisPort + Send + Sync>,
}

impl AuthUseCase {
    pub fn new(
        adapter: Arc<dyn DbPort<User> + Send + Sync>,
        session: Arc<dyn DbPort<UserSessions> + Send + Sync>,
        redis_adapter: Arc<dyn RedisPort + Send + Sync>,
    ) -> Self {
        AuthUseCase {
            adapter,
            session,
            redis_adapter,
        }
    }
}

#[tonic::async_trait]
impl AuthHandler for AuthUseCase {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let register_req = request.into_inner();

        let hashed_password = hash(&register_req.password, DEFAULT_COST).map_err(|_| {
            error!("Failed to hash password");
            Status::internal("Failed to hash password")
        })?;

        let user = User::new(register_req.email, hashed_password, UserStatus::Active);

        self.adapter.save(&user).await.map_err(|e| {
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
        let ip = get_client_ip(&request).ok_or_else(|| {
            error!("Failed to get client IP");
            Status::internal("Failed to get client IP")
        })?;

        let device = get_device_info(&request).ok_or_else(|| {
            error!("Failed to get device info");
            Status::internal("Failed to get device info")
        })?;

        let location = match get_location(&ip).await {
            Some(loc) => loc,
            None => {
                error!("Failed to get geolocation for IP: {}", ip);
                GeoLocation {
                    city: String::new(),
                    country: String::new(),
                    region: String::new(),
                    latitude: 0.0,
                    longitude: 0.0,
                }
            }
        };

        let login_req = request.into_inner();

        if let Some(user) = self
            .adapter
            .find_by_coll("email", &login_req.email)
            .await
            .map_err(|_| {
                error!("Failed to query user with email: {}", login_req.email);
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

                self.redis_adapter
                    .set_value(&access_token, &user_json)
                    .await
                    .expect("Failed to set value in Redis at Login");
                info!("Redis set value for user: {}", user_json);

                let response = LoginResponse {
                    access_token,
                    refresh_token,
                };

                let user_session = UserSessions::new(
                    user.id,
                    ip.clone().parse().expect("Invalid IP address"),
                    device.clone(),
                    serde_json::to_string(&location).unwrap_or_default(),
                );

                self.session.save(&user_session).await.map_err(|e| {
                    error!("Failed to save user session: {}", e);
                    Status::internal("Failed to save user session")
                })?;

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
        validate_access_token(&metadata, &self.redis_adapter).await?;

        let access_token = extract_token_from_metadata(&metadata)
            .map_err(|e| {
                error!("Failed to extract token: {}", e);
                Status::unauthenticated("Invalid token")
            })?
            .to_string();

        let logout_req = request.into_inner();
        let refresh_token = &logout_req.refresh_token;

        self.redis_adapter
            .ensure_not_blacklisted(refresh_token)
            .await?;

        let config = cfg();
        Token::validate_token(refresh_token, &config.refresh_secret).map_err(|e| {
            error!("Invalid refresh token: {}", e);
            Status::unauthenticated("Invalid refresh token")
        })?;

        self.redis_adapter
            .blacklist_token(refresh_token)
            .await
            .map_err(|e| {
                error!("Failed to blacklist refresh token: {}", e);
                Status::internal("Logout failed")
            })?;

        self.redis_adapter.blacklist_token(&access_token).await.ok();

        info!("Logout success: {}", refresh_token);

        Ok(Response::new(LogoutResponse {
            message: "Logout successful".to_string(),
        }))
    }
}
