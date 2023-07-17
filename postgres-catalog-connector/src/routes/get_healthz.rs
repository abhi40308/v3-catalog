use axum::response::{IntoResponse, Response};

pub const ROUTENAME: &str = "/healthz";

#[axum_macros::debug_handler()]
pub async fn handler() -> Response {
    println!("received healthz request");
    String::from("OK").into_response()
}
