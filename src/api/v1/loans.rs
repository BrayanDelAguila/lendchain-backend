use actix_web::{web, HttpResponse};

use crate::errors::AppError;

// TODO: implementar handlers de préstamos

/// GET /api/v1/loans
pub async fn list_loans() -> Result<HttpResponse, AppError> {
    // TODO: implementar paginación y filtros
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": [],
        "message": "TODO: implementar"
    })))
}

/// POST /api/v1/loans
pub async fn create_loan() -> Result<HttpResponse, AppError> {
    // TODO: validar body, calcular cuotas, crear préstamo
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": "TODO: implementar"
    })))
}

/// GET /api/v1/loans/{id}
pub async fn get_loan(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _id = path.into_inner();
    // TODO: buscar préstamo por id
    Err(AppError::NotFound)
}

/// POST /api/v1/loans/{id}/fund
pub async fn fund_loan(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _id = path.into_inner();
    // TODO: transferir fondos USDC vía blockchain
    Err(AppError::NotFound)
}

/// Configure loan routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/loans")
            .route("", web::get().to(list_loans))
            .route("", web::post().to(create_loan))
            .route("/{id}", web::get().to(get_loan))
            .route("/{id}/fund", web::post().to(fund_loan)),
    );
}
