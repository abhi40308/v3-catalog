use axum::Json;
use cc_postgres::configuration;
use cc_postgres::{error::ServerError, sql};
use ndc_client::models::{Argument, QueryRequest, QueryResponse};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    PgPool, Row,
};
use std::collections::BTreeMap;

pub const ROUTENAME: &str = "/query";

#[axum_macros::debug_handler]
pub async fn handler(
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, ServerError> {
    println!("received query request");
    resolve_query_request(&request).await
}

async fn resolve_query_request(request: &QueryRequest) -> Result<Json<QueryResponse>, ServerError> {
    // unwrap the variables from Option type; default to BTreeMap
    let vars = request.variables.clone().unwrap_or(vec![BTreeMap::new()]);

    // get the database_url variable from arguments or variables
    let database_url = get_argument_value(&request.arguments, &vars, "database_url".into());

    // create a postgres connection with DB URL
    // todo: brute force code - needs improvement
    let maybe_pool = match database_url {
        Some(db_url) => match db_url {
            serde_json::Value::String(val) => get_sql_connection_pool(val).await,
            _ => return Err(ServerError::BadRequest("invalid db url".into())),
        },
        None => match configuration::get_default_db_url() {
            Some(db_url) => get_sql_connection_pool(&db_url).await,
            None => return Err(ServerError::BadRequest("no db url provided".into())),
        },
    }
    .map_err(|err| {
        ServerError::BadRequest(format!(
            "could not connect to the given database: {}",
            err.to_string()
        ))
    });
    // throw an error if there was an error connecting to DB
    let pool = match maybe_pool {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    // build the SQL statement from request
    let sql_statement = sql::build_sql_query(request);

    // execute SQL, build response and return
    match sql_statement {
        Ok(statement) => {
            let result: PgRow = sqlx::query(statement.to_string().as_str())
                .fetch_one(&pool)
                .await?;
            let value: Result<sqlx::types::JsonValue, ServerError> = result
                .try_get(0)
                .map_err(|err| ServerError::DatabaseError(err.to_string()));
            // this parsing is technically not necessary, but useful for validation during development, and to ensure correctness.
            // todo: remove this, and instead send the first column of the first row as response without parsing or allocating additonal memory
            match value {
                Ok(v) => {
                    let response: Result<Json<QueryResponse>, ServerError> =
                        serde_json::from_value(v)
                            .map(|r| Json(r))
                            .map_err(|err| ServerError::Internal(err.to_string()));
                    response
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

// gets the PG connection pool to execute upon
// It's a single connection, so this can possibly be improved this by using PgConnection instead of PgPool
async fn get_sql_connection_pool(db_url: &String) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .connect(db_url)
        .await
}

// get's an argument value from arguments and variables provided in the request
// the borrowing can be improved
fn get_argument_value<'a>(
    arguments: &'a BTreeMap<String, Argument>,
    variables: &'a Vec<BTreeMap<String, serde_json::Value>>,
    key: String,
) -> Option<&'a serde_json::Value> {
    let argument = arguments.get(key.as_str());

    match argument {
        Some(arg) => match arg {
            Argument::Variable { name } => {
                let val = variables
                    .iter()
                    .find(|p| p.get(name).is_some())
                    .map(|vars| vars.get(name))
                    .and_then(|v| v);
                val
            }
            Argument::Literal { value } => Some(value),
        },
        None => None,
    }
}
