use crate::domain::validator::ValidateFromRequest;
use crate::impl_from_request;
use crate::pb::auth::{LoginRequest, LogoutRequest, RegisterRequest};
use validator::{Validate, ValidationError};

fn validate_password(pw: &str) -> bool {
    if pw.len() < 8 {
        return false;
    }
    let has_lower = pw.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = pw.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = pw.chars().any(|c| c.is_ascii_digit());
    let has_symbol = pw.chars().any(|c| !c.is_ascii_alphanumeric());

    has_lower && has_upper && has_digit && has_symbol
}

fn password_validator(password: &str) -> Result<(), ValidationError> {
    if !validate_password(password) {
        return Err(ValidationError::new("password_complexity"));
    }
    Ok(())
}

#[derive(Debug, Validate)]
pub struct RegisterDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(
        length(min = 8, message = "Password must be at least 8 characters long"),
        custom(
            function = "password_validator",
            message = "Password must have at least 8 characters, uppercase, lowercase, number, and special character"
        )
    )]
    pub password: String,
}

#[derive(Debug, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,

    #[validate(
        length(min = 8, message = "Password must be at least 8 characters long"),
        custom(
            function = "password_validator",
            message = "Password must have at least 8 characters, uppercase, lowercase, number, and special character"
        )
    )]
    pub password: String,
}

#[derive(Debug, Validate)]
pub struct LogoutDto {
    #[validate(length(min = 1, message = "Token cannot be empty"))]
    pub refresh_token: String,
}

impl_from_request!(RegisterDto, RegisterRequest, { email, password });
impl_from_request!(LoginDto, LoginRequest, { email, password });
impl_from_request!(LogoutDto, LogoutRequest, { refresh_token });
