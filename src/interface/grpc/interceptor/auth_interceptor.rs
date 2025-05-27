use crate::cfg;
use crate::domain::port::redis_port::RedisPort;
use crate::domain::service::jwt_service::Token;
use std::sync::Arc;
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

pub async fn authenticate_interceptor(
    req: Request<()>,
    redis_port: &Arc<dyn RedisPort + Send + Sync>,
) -> Result<Request<()>, Status> {
    let token = extract_token_from_metadata(req.metadata())?;

    redis_port.ensure_not_blacklisted(token).await?;

    let config = cfg();
    match Token::validate_token(token, &config.access_secret) {
        Ok(_) => Ok(req),
        Err(status) => Err(status),
    }
}

pub async fn validate_access_token(
    metadata: &tonic::metadata::MetadataMap,
    redis_port: &Arc<dyn RedisPort + Send + Sync>,
) -> Result<(), Status> {
    let token = extract_token_from_metadata(metadata)?;

    redis_port.ensure_not_blacklisted(token).await?;

    let config = cfg();
    match Token::validate_token(token, &config.access_secret) {
        Ok(_) => Ok(()),
        Err(status) => Err(status),
    }
}
