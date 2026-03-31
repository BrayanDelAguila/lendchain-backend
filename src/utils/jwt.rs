/// JWT utilities — access token generation and verification.
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Claims embedded in every access token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject — user id as string.
    pub sub: String,
    pub email: String,
    pub role: String,
    /// Unix timestamp: expiration.
    pub exp: usize,
    /// Unix timestamp: issued-at.
    pub iat: usize,
}

/// Generate an access token with a 15-minute lifetime.
pub fn generate_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    secret: &str,
) -> anyhow::Result<String> {
    let now = Utc::now();
    let exp = (now + chrono::Duration::minutes(15)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_owned(),
        role: role.to_owned(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("Token generation failed: {}", e))
}

/// Generate an opaque refresh token (random UUID v4 as a string).
pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

/// Verify an access token and return its claims.
pub fn verify_access_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| anyhow::anyhow!("Token verification failed: {}", e))?;

    Ok(token_data.claims)
}
