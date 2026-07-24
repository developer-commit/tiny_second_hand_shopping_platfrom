use axum::{
    http::StatusCode,
    response::{IntoResponse, Response, Json},
};
use shared::dto::error_dto::ApiErrorRes;

pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    BadGateway(String),
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_res) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ApiErrorRes::new("BAD_REQUEST", msg)),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, ApiErrorRes::new("UNAUTHORIZED", "Authentication required".to_string())),
            AppError::Forbidden => (StatusCode::FORBIDDEN, ApiErrorRes::new("FORBIDDEN", "You do not have permission".to_string())),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ApiErrorRes::new("NOT_FOUND", msg)),
            AppError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, ApiErrorRes::new("BAD_GATEWAY", msg)),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, ApiErrorRes::new("INTERNAL_ERROR", "An internal error occurred".to_string())),
        };

        (status, Json(error_res)).into_response()
    }
}
