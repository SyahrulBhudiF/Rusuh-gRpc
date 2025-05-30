use crate::cfg;
use crate::domain::dto::auth_dto::{LoginDto, LogoutDto, RegisterDto};
use crate::domain::entity::user::{User, UserStatus};
use crate::domain::entity::user_sessions::UserSessions;
use crate::domain::port::db_port::DbPort;
use crate::domain::port::redis_port::RedisPort;
use crate::domain::service::jwt_service::Token;
use crate::interface::common::client_info::GeoLocation;
use crate::pb::auth::{LoginResponse, LogoutResponse, RegisterResponse, User as UserResponse};
use crate::util::util::{hash_password_async, verify_password_async};
use std::sync::Arc;
use tonic::{Response, Status};
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

impl AuthUseCase {
    pub(crate) async fn register(
        &self,
        request: RegisterDto,
    ) -> Result<Response<RegisterResponse>, Status> {
        let user_exists = self
            .adapter
            .find_by_coll("email", &request.email)
            .await
            .map_err(|_| {
                error!("Failed to query user with email: {}", request.email);
                Status::internal("Failed to query user")
            })?;

        if user_exists.is_some() {
            error!("User with email {} already exists", request.email);
            return Err(Status::already_exists("User already exists"));
        }

        let hashed_password = hash_password_async(request.password).await.map_err(|e| {
            error!("Failed to hash password: {}", e);
            Status::internal("Failed to hash password")
        })?;

        let user = User::new(
            request.name,
            request.email,
            hashed_password,
            UserStatus::Active,
        );

        self.adapter.save(&user).await.map_err(|e| {
            error!("Failed to save user: {}", e);
            Status::internal(format!("Failed to save user: {}", e))
        })?;

        let proto_user = UserResponse {
            id: user.id.to_string(),
            email: user.email.clone(),
        };

        let response = RegisterResponse {
            user: Some(proto_user),
        };

        info!("User registered successfully: {}", user.email);
        Ok(Response::new(response))
    }

    pub(crate) async fn login(
        &self,
        request: LoginDto,
        ip: String,
        device: String,
        location: GeoLocation,
    ) -> Result<Response<LoginResponse>, Status> {
        let login_req = request;

        if let Some(user) = self
            .adapter
            .find_by_coll("email", &login_req.email)
            .await
            .map_err(|_| {
                error!("Failed to query user with email: {}", login_req.email);
                Status::not_found("Failed to query user")
            })?
        {
            let password_valid = verify_password_async(&login_req.password, &user.password).await;

            return if let Ok(true) = password_valid {
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
                if let Err(e) = password_valid {
                    error!("Failed to verify password: {}", e);
                } else {
                    error!("Invalid password or email for user: {}", login_req.email);
                }

                Err(Status::unauthenticated("Invalid password or email"))
            };
        }

        error!("Invalid email or password for user: {}", login_req.email);
        Err(Status::unauthenticated("Invalid email or password"))
    }

    pub(crate) async fn logout(
        &self,
        request: LogoutDto,
        access_token: String,
    ) -> Result<Response<LogoutResponse>, Status> {
        let logout_req = request;
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
