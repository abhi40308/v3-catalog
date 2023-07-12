use axum::{routing::get, routing::post, Router};

mod get_capabilities;
mod get_healthz;
mod get_schema;
mod post_query;
mod post_query_explain;

pub fn create_router() -> Router {
    Router::new()
        .route(get_healthz::ROUTENAME, get(get_healthz::handler))
        .route(get_capabilities::ROUTENAME, get(get_capabilities::handler))
        .route(get_schema::ROUTENAME, get(get_schema::handler))
        .route(
            post_query_explain::ROUTENAME,
            post(post_query_explain::handler),
        )
        .route(post_query::ROUTENAME, post(post_query::handler))
}
