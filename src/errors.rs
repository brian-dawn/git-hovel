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
    InternalServerError(Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for HovelError {
    fn into_response(self) -> Response {
        let status = match &self {
            HovelError::NotFound => StatusCode::NOT_FOUND,
            HovelError::BadRequest => StatusCode::BAD_REQUEST,
            HovelError::InternalServerError(e) => StatusCode::INTERNAL_SERVER_ERROR,
            // Handle other errors
        };

        tracing::error!("{:?}", &self);

        (status, "oops").into_response()
    }
}

impl From<sqlx::Error> for HovelError {
    fn from(e: sqlx::Error) -> Self {
        Self::InternalServerError(Box::new(e))
    }
}

// Now for askama errors.
impl From<askama::Error> for HovelError {
    fn from(e: askama::Error) -> Self {
        Self::InternalServerError(Box::new(e))
    }
}

impl From<russh::Error> for HovelError {
    fn from(e: russh::Error) -> Self {
        Self::InternalServerError(Box::new(e))
    }
}

