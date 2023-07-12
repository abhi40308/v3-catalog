use axum::Json;
use ndc_client::models::{
    Capabilities, CapabilitiesResponse, QueryCapabilities,
};

pub const ROUTENAME: &str = "/capabilities";

#[axum_macros::debug_handler]
pub async fn handler() -> Json<CapabilitiesResponse> {
    println!("received capabilities request");
    let empty: serde_json::Value = serde_json::from_str("{}").unwrap();
    Json(CapabilitiesResponse {
        versions: "^1.0.0".into(),
        capabilities: Capabilities {
            explain: Some(empty.clone()),
            query: Some(QueryCapabilities {
                foreach: None,
                order_by_aggregate: None,
                relation_comparisons: None,
            }),
            mutations: None,
            relationships: Some(empty),
        },
    })
}