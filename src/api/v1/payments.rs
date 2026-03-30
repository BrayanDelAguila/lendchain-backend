use actix_web::{web, HttpResponse};

use crate::errors::AppError;

// TODO: implementar handlers de pagos

/// GET /api/v1/loans/{loan_id}/payments
pub async fn list_payments(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _loan_id = path.into_inner();
    // TODO: listar cuotas del préstamo
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": [],
        "message": "TODO: implementar"
    })))
}

/// POST /api/v1/loans/{loan_id}/payments
pub async fn make_payment(path: web::Path<uuid::Uuid>) -> Result<HttpResponse, AppError> {
    let _loan_id = path.into_inner();
    // TODO: registrar pago en blockchain y db
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": "TODO: implementar"
    })))
}

/// Configure payment routes (nested under loans).
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/loans/{loan_id}/payments")
            .route("", web::get().to(list_payments))
            .route("", web::post().to(make_payment)),
    );
}
