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

/// GET /api/v1/loans/available
pub async fn list_available_loans() -> Result<HttpResponse, AppError> {
    // TODO: listar préstamos en estado PENDING disponibles para fondear
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

/// GET /api/v1/loans/{id}/schedule
pub async fn get_loan_schedule(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _id = path.into_inner();
    // TODO: retornar tabla de amortización del préstamo usando utils::calculator
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": [],
        "message": "TODO: implementar"
    })))
}

/// POST /api/v1/loans/{id}/fund
pub async fn fund_loan(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _id = path.into_inner();
    // TODO: transferir fondos USDC vía blockchain
    Err(AppError::NotFound)
}

/// POST /api/v1/loans/{id}/pay
pub async fn pay_loan_installment(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _id = path.into_inner();
    // TODO: registrar pago de cuota en blockchain y marcar payment como CONFIRMED
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": "TODO: implementar"
    })))
}

/// Configure loan routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/loans")
            .route("", web::get().to(list_loans))
            .route("", web::post().to(create_loan))
            .route("/available", web::get().to(list_available_loans))
            .route("/{id}", web::get().to(get_loan))
            .route("/{id}/schedule", web::get().to(get_loan_schedule))
            .route("/{id}/fund", web::post().to(fund_loan))
            .route("/{id}/pay", web::post().to(pay_loan_installment)),
    );
}
