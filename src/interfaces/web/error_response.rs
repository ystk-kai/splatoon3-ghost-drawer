use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

impl ErrorResponse {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            error: status_code
                .canonical_reason()
                .unwrap_or("Unknown Error")
                .to_string(),
            message: message.into(),
            status_code: status_code.as_u16(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status_code, Json(self)).into_response()
    }
}
