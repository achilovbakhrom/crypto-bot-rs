use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")] Database(#[from] sea_orm::DbErr),

    #[error("Encryption error: {0}")] Encryption(String),

    #[error("Invalid input: {0}")] InvalidInput(String),

    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Chain error: {0}")] Chain(String),

    #[error("RPC error: {0}")] Rpc(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid address")]
    InvalidAddress,

    #[error("Invalid mnemonic")]
    InvalidMnemonic,

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Configuration error: {0}")] Config(String),

    #[error("Internal error: {0}")] Internal(String),
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(serde::Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl AppError {
    pub fn to_error_response(&self) -> ErrorResponse {
        let (code, message, field) = match self {
            AppError::Database(e) => ("DATABASE_ERROR", e.to_string(), None),
            AppError::Encryption(msg) => ("ENCRYPTION_ERROR", msg.clone(), None),
            AppError::InvalidInput(msg) => ("INVALID_INPUT", msg.clone(), None),
            AppError::WalletNotFound => ("WALLET_NOT_FOUND", "Wallet not found".to_string(), None),
            AppError::Chain(msg) => ("CHAIN_ERROR", msg.clone(), None),
            AppError::Rpc(msg) => ("RPC_ERROR", msg.clone(), None),
            AppError::InsufficientBalance =>
                ("INSUFFICIENT_BALANCE", "Insufficient balance for transaction".to_string(), None),
            AppError::InvalidAddress =>
                (
                    "INVALID_ADDRESS",
                    "Invalid address format".to_string(),
                    Some("address".to_string()),
                ),
            AppError::InvalidMnemonic =>
                (
                    "INVALID_MNEMONIC",
                    "Invalid mnemonic phrase".to_string(),
                    Some("mnemonic".to_string()),
                ),
            AppError::InvalidPrivateKey =>
                (
                    "INVALID_PRIVATE_KEY",
                    "Invalid private key format".to_string(),
                    Some("private_key".to_string()),
                ),
            AppError::Config(msg) => ("CONFIG_ERROR", msg.clone(), None),
            AppError::Internal(msg) => ("INTERNAL_ERROR", msg.clone(), None),
        };

        ErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                field,
            },
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::WalletNotFound => axum::http::StatusCode::NOT_FOUND,
            | AppError::InvalidInput(_)
            | AppError::InvalidAddress
            | AppError::InvalidMnemonic
            | AppError::InvalidPrivateKey => {
                axum::http::StatusCode::BAD_REQUEST
            }
            AppError::InsufficientBalance => axum::http::StatusCode::BAD_REQUEST,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let response = self.to_error_response();
        (status, axum::Json(response)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
