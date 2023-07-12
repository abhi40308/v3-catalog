use axum::Json;
use cc_postgres::sql;

use cc_postgres::error::ServerError;
use ndc_client::models::{ExplainResponse, QueryRequest};

pub const ROUTENAME: &str = "/query/explain";

#[axum_macros::debug_handler]
pub async fn handler(
    Json(request): Json<QueryRequest>,
) -> Result<Json<ExplainResponse>, ServerError> {

    println!("received query explain request");

    let query = sql::build_sql_query(&request);
    let built_query = match query {
    	Ok(q) => q,
    	Err(err) => return Err(err)
    };

    let response = ExplainResponse {
        lines: vec![],
        query: built_query.to_string(),
    };

    Ok(Json(response))
}

/*
curl -d '{ "table": "test", "query": { "limit": 10, "fields": { "table_name": { "type": "column", "column": "table_name", "arguments": {} }, "table_schema": { "type": "column", "column": "table_schema", "arguments": {} } } }, "arguments": {}, "table_relationships": {} }' -H "Content-Type: application/json" -X POST http://localhost:3000/query/explain
*/