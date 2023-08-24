use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct YaissError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for YaissError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("YaissError: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for YaissError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
