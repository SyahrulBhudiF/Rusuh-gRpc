use crate::domain::jwt::Token;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::pb::auth::auth_service_server::AuthService;
use crate::pb::auth::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, User as ProtoUser,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct AuthServiceImpl {
    user_repo: Box<dyn Repository<User>>,
}

impl AuthServiceImpl {
    pub fn new(user_repo: Box<dyn Repository<User>>) -> Self {
        AuthServiceImpl { user_repo }
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

        error!("Invalid email or password");
        Err(Status::unauthenticated("Invalid email or password"))
    }
}
