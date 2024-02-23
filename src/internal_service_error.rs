use std::fmt::{Debug, Display};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppError(color_eyre::eyre::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("<h1>Whoops!</h1> <p>Something went wrong: {}</p>", self.0),
        )
            .into_response()
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E> From<E> for AppError
where
    E: Into<color_eyre::eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
