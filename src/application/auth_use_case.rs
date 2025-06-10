use crate::domain::dto::auth_dto::{
    ForgotPasswordDto, LoginDto, LogoutDto, RegisterDto, SendOtpDto, VerifyEmailDto,
};
use crate::domain::entity::user::{User, UserStatus};
use crate::domain::entity::user_sessions::UserSessions;
use crate::domain::port::db::user_port::UserPort;
use crate::domain::port::db_port::DbPort;
use crate::domain::port::redis_port::RedisPort;
use crate::domain::service::jwt_service::Token;
use crate::interface::common::client_info::GeoLocation;
use crate::pb::auth::{
    ForgotPasswordResponse, LoginData, LoginResponse, LogoutResponse, RegisterData,
    RegisterResponse, SendOtpResponse, User as UserResponse, VerifyEmailResponse,
};
use crate::util::util::{hash_password_async, verify_password_async};
use crate::{cfg, email, email_otp};
use std::sync::Arc;
use tonic::{Response, Status};
use tracing::{error, info};

pub struct AuthUseCase {
    adapter: Arc<dyn UserPort + Send + Sync>,
    session: Arc<dyn DbPort<UserSessions> + Send + Sync>,
    redis_adapter: Arc<dyn RedisPort + Send + Sync>,
}

impl AuthUseCase {
    pub fn new(
        adapter: Arc<dyn UserPort + Send + Sync>,
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
            UserStatus::Inactive,
        );

        self.adapter.save(&user).await.map_err(|e| {
            error!("Failed to save user: {}", e);
            Status::internal(format!("Failed to save user: {}", e))
        })?;

        let proto_user = UserResponse {
            id: user.id.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
        };

        let response = RegisterResponse {
            message: "User registered successfully".to_string(),
            data: Some(RegisterData {
                user: Some(proto_user),
            }),
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
            if user.status != UserStatus::Active {
                error!("User with email {} is not active", login_req.email);
                return Err(Status::permission_denied("Verify your email first"));
            }

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
                    message: "Login successful".to_string(),
                    data: Some(LoginData {
                        access_token,
                        refresh_token,
                    }),
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

    pub(crate) async fn send_otp(
        &self,
        request: SendOtpDto,
    ) -> Result<Response<SendOtpResponse>, Status> {
        let existing_user = self
            .adapter
            .find_by_coll("email", &request.email)
            .await
            .map_err(|e| {
                error!(
                    "Database error during user lookup for email {}: {}",
                    request.email, e
                );
                Status::internal("Failed to lookup user")
            })?;

        if existing_user.is_none() {
            error!("User with email {} does not exist", request.email);
            return Err(Status::not_found("User not found"));
        }

        let otp_helper = email_otp();

        let otp_code = otp_helper.generate_code(6);
        let otp_key = format!("otp:{}", request.email);
        self.redis_adapter
            .set_value(&otp_key, &otp_code)
            .await
            .map_err(|e| {
                error!("Failed to set OTP in Redis: {}", e);
                Status::internal("Failed to set OTP")
            })?;

        info!("OTP generated for user: {}", request.email);

        let email_bg = request.email.clone();

        tokio::spawn(async move {
            let email_sender = email();
            match email_sender.send_otp_email(&email_bg, &otp_code).await {
                Ok(_) => info!("Background OTP email sent successfully to: {}", email_bg),
                Err(e) => error!("Background OTP email failed for {}: {}", email_bg, e),
            }
        });

        info!(
            "OTP request response sent immediately for: {}",
            request.email
        );
        Ok(Response::new(SendOtpResponse {
            message: "OTP request sent successfully".to_string(),
        }))
    }

    pub(crate) async fn verify_email(
        &self,
        request: VerifyEmailDto,
    ) -> Result<Response<VerifyEmailResponse>, Status> {
        let existing_user = self
            .adapter
            .find_by_coll("email", &request.email)
            .await
            .map_err(|e| {
                error!(
                    "Database error during user lookup for email {}: {}",
                    request.email, e
                );
                Status::internal("Failed to lookup user")
            })?;

        if existing_user.is_none() {
            error!("User with email {} does not exist", request.email);
            return Err(Status::not_found("User not found"));
        }

        let otp_key = format!("otp:{}", request.email);
        let existing_otp = self.redis_adapter.get_value(&otp_key).await.map_err(|e| {
            error!("Failed to get OTP from Redis: {}", e);
            Status::internal("Failed to get OTP")
        })?;

        if existing_otp.is_none() || existing_otp.unwrap() != request.otp {
            error!(
                "OTP verification failed for email {}: Invalid code",
                request.email
            );
            return Err(Status::invalid_argument("Invalid OTP code"));
        }

        let user = existing_user.unwrap();

        self.adapter.verify_email(user.id).await.map_err(|e| {
            error!("Failed to verify email for user {}: {}", user.email, e);
            Status::internal("Failed to verify email")
        })?;

        self.redis_adapter
            .delete_value(&otp_key)
            .await
            .map_err(|e| {
                error!("Failed to remove OTP from Redis: {}", e);
                Status::internal("Failed to remove OTP")
            })?;

        info!("OTP verified successfully for email: {}", request.email);

        Ok(Response::new(VerifyEmailResponse {
            message: "Email verified successfully".to_string(),
        }))
    }

    pub(crate) async fn forgot_password(
        &self,
        request: ForgotPasswordDto,
    ) -> Result<Response<ForgotPasswordResponse>, Status> {
        let existing_user = self
            .adapter
            .find_by_coll("email", &request.email)
            .await
            .map_err(|e| {
                error!(
                    "Database error during user lookup for email {}: {}",
                    request.email, e
                );
                Status::internal("Failed to lookup user")
            })?;

        if existing_user.is_none() {
            error!("User with email {} does not exist", request.email);
            return Err(Status::not_found("User not found"));
        }

        let otp_key = format!("otp:{}", request.email);
        let existing_otp = self.redis_adapter.get_value(&otp_key).await.map_err(|e| {
            error!("Failed to get OTP from Redis: {}", e);
            Status::internal("Failed to get OTP")
        })?;

        if existing_otp.is_none() || existing_otp.unwrap() != request.otp {
            error!(
                "OTP verification failed for email {}: Invalid code",
                request.email
            );
            return Err(Status::invalid_argument("Invalid OTP code"));
        }

        let mut user = existing_user.unwrap();
        let hashed_password = hash_password_async(request.password).await.map_err(|e| {
            error!("Failed to hash new password: {}", e);
            Status::internal("Failed to hash new password")
        })?;

        user.password = hashed_password;

        self.adapter.update(user.id, &user).await.map_err(|e| {
            error!("Failed to update password for user {}: {}", user.email, e);
            Status::internal("Failed to update password")
        })?;

        self.redis_adapter
            .delete_value(&otp_key)
            .await
            .map_err(|e| {
                error!("Failed to remove OTP from Redis: {}", e);
                Status::internal("Failed to remove OTP")
            })?;

        info!("Password reset successfully for email: {}", request.email);
        Ok(Response::new(ForgotPasswordResponse {
            message: "Password reset successfully".to_string(),
        }))
    }
}
