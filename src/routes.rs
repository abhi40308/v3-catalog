use axum::{
    routing::get,
    Router,
};

mod healthz;

pub fn create_router() -> Router {
    Router::new()
        .route(healthz::ROUTENAME, get(healthz::handler))
}