use crate::domain::jwt::{AuthClaims, get_jwt_secret};
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::pb::auth::auth_service_server::AuthService;
use crate::pb::auth::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, User as ProtoUser,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use std::time::{Duration, SystemTime};
use tonic::{Request, Response, Status};

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

        let hashed_password = hash(&register_req.password, DEFAULT_COST).unwrap();

        let user = User::new(register_req.email.clone(), hashed_password);

        self.user_repo
            .save(&user)
            .await
            .map_err(|e| Status::internal(format!("Failed to save user: {}", e)))?;

        let proto_user = ProtoUser {
            id: user.id.to_string(),
            email: user.email,
        };

        let response = RegisterResponse {
            user: Some(proto_user),
        };

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
            .unwrap()
        {
            if verify(&login_req.password, &user.password).unwrap() {
                let expiration = SystemTime::now() + Duration::new(3600, 0);
                let claims = AuthClaims::new(user.id.to_string(), expiration);

                let token = encode(
                    &Header::new(Algorithm::HS256),
                    &claims,
                    &EncodingKey::from_secret(get_jwt_secret().as_ref()),
                )
                .unwrap();

                let response = LoginResponse {
                    access_token: token,
                };
                return Ok(Response::new(response));
            }
        }

        Err(Status::unauthenticated("Invalid email or password"))
    }
}
