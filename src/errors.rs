use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HovelError {
    #[error("Not Found")]
    NotFound,

    #[error("Bad Request")]
    BadRequest,

    #[error("Internal Server Error")]
    InternalServerError,
}

impl IntoResponse for HovelError {
    fn into_response(self) -> Response {
        let status = match self {
            HovelError::NotFound => StatusCode::NOT_FOUND,
            HovelError::BadRequest => StatusCode::BAD_REQUEST,
            HovelError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            // Handle other errors
        };

        (status, self.to_string()).into_response()
    }
}
