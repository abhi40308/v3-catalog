mod configuration;
mod routes;
use axum;

#[tokio::main]
async fn main() {
    // initialize routes
    let router = routes::create_router();

    // get and print server address
    let server_config = configuration::get_configuration();
    let address = format!("0.0.0.0:{}", server_config.port);
    println!("Starting server at {}", address);

    // listen
    axum::Server::bind(&address.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
