/// Actix-web extractor that validates JWT and requires role = 'ADMIN'.
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use std::future::{ready, Ready};

use crate::config::Config;
use crate::errors::AppError;
use crate::utils::jwt::{verify_access_token, Claims};

pub struct AdminUser(pub Claims);

impl FromRequest for AdminUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(extract_admin_claims(req))
    }
}

fn extract_admin_claims(req: &HttpRequest) -> Result<AdminUser, AppError> {
    let config = req
        .app_data::<web::Data<Config>>()
        .ok_or(AppError::Unauthorized)?;

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let claims =
        verify_access_token(token, &config.jwt_secret).map_err(|_| AppError::Unauthorized)?;

    if claims.role != "ADMIN" {
        return Err(AppError::Forbidden);
    }

    Ok(AdminUser(claims))
}
