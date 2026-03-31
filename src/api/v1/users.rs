use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;
use uuid::Uuid;

use crate::config::Config;
use crate::db::users as db;
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::user_service::{self, LoginDto, RegisterDto};

// ─── Request bodies ───────────────────────────────────────────────────────────

#[derive(Deserialize, Validate)]
pub struct RegisterBody {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 2, message = "Full name must be at least 2 characters"))]
    pub full_name: String,
    pub document_number: Option<String>,
    pub phone: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/v1/users/register
pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<RegisterBody>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let dto = RegisterDto {
        email: body.email.clone(),
        password: body.password.clone(),
        full_name: body.full_name.clone(),
        document_number: body.document_number.clone(),
        phone: body.phone.clone(),
    };

    let auth = user_service::register(
        &pool,
        dto,
        &config.jwt_secret,
        &config.wallet_encryption_key,
    )
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "data": {
            "user": auth.user,
            "access_token": auth.access_token,
            "refresh_token": auth.refresh_token
        }
    })))
}

/// POST /api/v1/users/login
pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<LoginBody>,
) -> Result<HttpResponse, AppError> {
    let dto = LoginDto {
        email: body.email.clone(),
        password: body.password.clone(),
    };

    let auth = user_service::login(&pool, dto, &config.jwt_secret).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "user": auth.user,
            "access_token": auth.access_token,
            "refresh_token": auth.refresh_token
        }
    })))
}

/// GET /api/v1/users/me
pub async fn me(
    pool: web::Data<PgPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let user_id: Uuid = auth
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized)?;

    let user = db::find_by_id(&pool, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let public: crate::models::user::UserPublic = user.into();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": public
    })))
}

/// Configure user routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/me", web::get().to(me)),
    );
}
