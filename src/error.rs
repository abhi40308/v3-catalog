use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub enum ServerError {
    Internal(String),
    DatabaseError(String),
    BadRequest(String)
}

#[derive(Serialize)]
struct JsonErrorResponse {
    message: String,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServerError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ServerError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        println!("Returning error: {message} with status code: {status}");
        (status, Json(JsonErrorResponse { message })).into_response()
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(value: sqlx::Error) -> Self {
        ServerError::DatabaseError(value.to_string())
    }
}
