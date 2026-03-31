/// Actix-web extractor that validates a Bearer JWT and injects the claims.
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use std::future::{ready, Ready};

use crate::config::Config;
use crate::errors::AppError;
use crate::utils::jwt::{verify_access_token, Claims};

/// Extractor for authenticated requests.
///
/// Usage in a handler:
/// ```rust,ignore
/// pub async fn me(auth: AuthenticatedUser) -> Result<HttpResponse, AppError> {
///     println!("{}", auth.0.sub);
///     ...
/// }
/// ```
pub struct AuthenticatedUser(pub Claims);

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let result = extract_claims(req);
        ready(result)
    }
}

fn extract_claims(req: &HttpRequest) -> Result<AuthenticatedUser, AppError> {
    // Get jwt_secret from app_data
    let config = req
        .app_data::<web::Data<Config>>()
        .ok_or(AppError::Unauthorized)?;

    // Read Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    // Strip "Bearer " prefix
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    // Verify and decode
    let claims =
        verify_access_token(token, &config.jwt_secret).map_err(|_| AppError::Unauthorized)?;

    Ok(AuthenticatedUser(claims))
}
