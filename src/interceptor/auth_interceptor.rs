use crate::domain::jwt::Token;
use crate::domain::redis_repository::RedisRepository;
use crate::infrastructure::redis_repository::RedisRepositoryImpl;
use tonic::{Request, Status};

pub fn extract_token_from_metadata(
    metadata: &tonic::metadata::MetadataMap,
) -> Result<&str, Status> {
    let token = metadata
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("Authorization token is missing"))?;

    let token_str = token
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid token format"))?;

    if token_str.starts_with("Bearer ") {
        Ok(&token_str["Bearer ".len()..])
    } else {
        Err(Status::unauthenticated("Invalid token scheme"))
    }
}

pub async fn authenticate_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = extract_token_from_metadata(req.metadata())?;

    RedisRepositoryImpl::new()
        .ensure_not_blacklisted(token)
        .await?;

    match Token::validate_token(token, "ACCESS_SECRET") {
        Ok(_) => Ok(req),
        Err(status) => Err(status),
    }
}

pub async fn validate_access_token(metadata: &tonic::metadata::MetadataMap) -> Result<(), Status> {
    let token = extract_token_from_metadata(metadata)?;

    RedisRepositoryImpl::new()
        .ensure_not_blacklisted(token)
        .await?;

    match Token::validate_token(token, "ACCESS_SECRET") {
        Ok(_) => Ok(()),
        Err(status) => Err(status),
    }
}
