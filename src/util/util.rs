use bcrypt::{DEFAULT_COST, hash, verify};
use std::error::Error;

pub async fn hash_password_async(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let handle = tokio::task::spawn_blocking(move || hash(password, DEFAULT_COST));
    let result = handle.await.map_err(|e| format!("Join error: {}", e))??;
    Ok(result)
}

pub async fn verify_password_async(
    plain: &str,
    hashed: &str,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let plain = plain.to_string();
    let hashed = hashed.to_string();

    let result = tokio::task::spawn_blocking(move || verify(&plain, &hashed))
        .await
        .map_err(|e| format!("Join error: {}", e))??;

    Ok(result)
}
