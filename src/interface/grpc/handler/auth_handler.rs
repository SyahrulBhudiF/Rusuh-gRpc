use crate::application::auth_use_case::AuthUseCase;
use crate::domain::dto::auth_dto::{
    ForgotPasswordDto, LoginDto, LogoutDto, RegisterDto, SendOtpDto, VerifyEmailDto,
};
use crate::domain::entity::user_sessions::UserSessions;
use crate::domain::port::db::user_port::UserPort;
use crate::domain::port::db_port::DbPort;
use crate::domain::port::redis_port::RedisPort;
use crate::domain::validator::ValidateFromRequest;
use crate::interface::common::client_info::{
    GeoLocation, get_client_ip, get_device_info, get_location,
};
use crate::interface::grpc::interceptor::auth_interceptor::{
    extract_token_from_metadata, validate_access_token,
};
use crate::pb::auth::auth_handler_server::AuthHandler as Handler;
use crate::pb::auth::{
    ForgotPasswordRequest, ForgotPasswordResponse, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, RegisterRequest, RegisterResponse, SendOtpRequest,
};
use crate::pb::auth::{SendOtpResponse, VerifyEmailRequest, VerifyEmailResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::error;

pub struct AuthHandler {
    auth_service: AuthUseCase,
    redis_port: Arc<dyn RedisPort + Send + Sync>,
}

impl AuthHandler {
    pub fn new(
        port: Arc<dyn UserPort + Send + Sync>,
        session: Arc<dyn DbPort<UserSessions> + Send + Sync>,
        redis_port: Arc<dyn RedisPort + Send + Sync>,
    ) -> Self {
        AuthHandler {
            auth_service: AuthUseCase::new(port, session, redis_port.clone()),
            redis_port,
        }
    }
}

#[tonic::async_trait]
impl Handler for AuthHandler {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let dto = RegisterDto::validate_from_request(request)?;
        self.auth_service.register(dto).await
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

        let dto = LoginDto::validate_from_request(request)?;
        self.auth_service.login(dto, ip, device, location).await
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let metadata = request.metadata().clone();
        validate_access_token(&metadata, &self.redis_port).await?;

        let access_token = extract_token_from_metadata(&metadata)
            .map_err(|e| {
                error!("Failed to extract token: {}", e);
                Status::unauthenticated("Invalid token")
            })?
            .to_string();

        let req = LogoutDto::validate_from_request(request)?;
        self.auth_service.logout(req, access_token).await
    }

    async fn send_otp(
        &self,
        request: Request<SendOtpRequest>,
    ) -> Result<Response<SendOtpResponse>, Status> {
        let dto = SendOtpDto::validate_from_request(request)?;
        self.auth_service.send_otp(dto).await
    }

    async fn verify_email(
        &self,
        request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailResponse>, Status> {
        let dto = VerifyEmailDto::validate_from_request(request)?;
        self.auth_service.verify_email(dto).await
    }

    async fn forgot_password(
        &self,
        request: Request<ForgotPasswordRequest>,
    ) -> Result<Response<ForgotPasswordResponse>, Status> {
        let dto = ForgotPasswordDto::validate_from_request(request)?;
        self.auth_service.forgot_password(dto).await
    }
}
