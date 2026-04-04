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
            AppError::BlockchainTxFailed(msg) => {
                tracing::error!(error = %msg, "Blockchain transaction failed");
                if msg.contains("insufficient funds") || msg.contains("gas") {
                    "Fondos insuficientes para pagar el gas de la transacción".to_string()
                } else {
                    format!("La transacción blockchain falló: {}", msg)
                }
            }
            AppError::BlockchainTimeout => {
                "La blockchain no respondió a tiempo. Intenta de nuevo en unos minutos.".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::MessageBody, ResponseError};

    fn body_to_json(err: AppError) -> serde_json::Value {
        let resp = err.error_response();
        let (_, body) = resp.into_parts();
        let bytes = body
            .try_into_bytes()
            .expect("error response body should be complete bytes");
        serde_json::from_slice(&bytes).expect("error response body should be valid JSON")
    }

    #[test]
    fn test_not_found_status() {
        assert_eq!(
            AppError::NotFound.status_code(),
            actix_web::http::StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_unauthorized_status() {
        assert_eq!(
            AppError::Unauthorized.status_code(),
            actix_web::http::StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_validation_status() {
        assert_eq!(
            AppError::Validation("bad input".to_string()).status_code(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_blockchain_tx_failed_status() {
        assert_eq!(
            AppError::BlockchainTxFailed("revert".to_string()).status_code(),
            actix_web::http::StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn test_blockchain_timeout_status() {
        assert_eq!(
            AppError::BlockchainTimeout.status_code(),
            actix_web::http::StatusCode::GATEWAY_TIMEOUT
        );
    }

    #[test]
    fn test_error_response_json_shape() {
        let json = body_to_json(AppError::NotFound);
        assert_eq!(json["success"], false, "Response must have success: false");
        assert!(
            json["error"]["code"].is_string(),
            "Response must have error.code as a string"
        );
        assert!(
            json["error"]["message"].is_string(),
            "Response must have error.message as a string"
        );
    }

    #[test]
    fn test_internal_error_hides_details() {
        let secret = "hunter2_db_password";
        let json = body_to_json(AppError::Internal(anyhow::anyhow!(
            "DB connection failed: password={}",
            secret
        )));
        let message = json["error"]["message"].as_str().unwrap_or_default();
        assert!(
            !message.contains(secret),
            "Internal error detail must not be exposed in the response, got: {}",
            message
        );
        assert!(
            message.to_lowercase().contains("internal"),
            "Generic internal error message expected, got: {}",
            message
        );
    }
}
