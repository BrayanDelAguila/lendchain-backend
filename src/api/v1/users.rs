use actix_web::{web, HttpResponse};

use crate::errors::AppError;

// TODO: implementar handlers de usuarios

/// POST /api/v1/users/register
pub async fn register() -> Result<HttpResponse, AppError> {
    // TODO: validar body, crear usuario y wallet
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": "TODO: implementar"
    })))
}

/// POST /api/v1/users/login
pub async fn login() -> Result<HttpResponse, AppError> {
    // TODO: verificar credenciales, emitir JWT
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "TODO: implementar"
    })))
}

/// GET /api/v1/users/me
pub async fn me() -> Result<HttpResponse, AppError> {
    // TODO: extraer user del JWT, retornar perfil
    Err(AppError::Unauthorized)
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
