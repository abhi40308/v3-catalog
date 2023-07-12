use axum::Json;
use ndc_client::models::{
    Capabilities, CapabilitiesResponse, QueryCapabilities,
};

pub const ROUTENAME: &str = "/capabilities";

#[axum_macros::debug_handler]
pub async fn handler() -> Json<CapabilitiesResponse> {
    println!("received capabilities request");
    let empty = serde_json::to_value(()).unwrap();
    Json(CapabilitiesResponse {
        versions: "^1.0.0".into(),
        capabilities: Capabilities {
            explain: None,
            query: Some(QueryCapabilities {
                foreach: Some(empty.clone()),
                order_by_aggregate: Some(empty.clone()),
                relation_comparisons: Some(empty.clone()),
            }),
            mutations: None,
            relationships: Some(empty),
        },
    })
}