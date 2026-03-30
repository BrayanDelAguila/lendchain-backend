use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

/// Application-level error enum.
/// Each variant maps to a specific HTTP status code and JSON error response.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Blockchain transaction failed: {0}")]
    BlockchainTxFailed(String),

    #[error("Blockchain operation timed out")]
    BlockchainTimeout,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Machine-readable error code string included in the JSON response.
    fn error_code(&self) -> &'static str {
        match self {
            AppError::NotFound => "NOT_FOUND",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::InvalidState(_) => "INVALID_STATE",
            AppError::BlockchainTxFailed(_) => "BLOCKCHAIN_TX_FAILED",
            AppError::BlockchainTimeout => "BLOCKCHAIN_TIMEOUT",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::InvalidState(_) => StatusCode::CONFLICT,
            AppError::BlockchainTxFailed(_) => StatusCode::BAD_GATEWAY,
            AppError::BlockchainTimeout => StatusCode::GATEWAY_TIMEOUT,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        // Do not expose internal details in production — log them instead.
        let message = match self {
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                "A database error occurred".to_string()
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "Internal error");
                "An internal error occurred".to_string()
            }
            other => other.to_string(),
        };

        HttpResponse::build(status).json(json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": message
            }
        }))
    }
}
