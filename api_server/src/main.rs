#[macro_use] extern crate serde_json;

use async_std::sync::Arc;
use mongodb::{options::ClientOptions, Client};
use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};

mod routes;

#[derive(Clone, Debug)]
pub struct State {
    client: Arc<Client>
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {

    // Configuring DB connection
    let mut client_options = match ClientOptions::parse("mongodb://localhost:27017").await {
        Ok(c) => c,
        Err(e) => panic!("Client Options Failed: {}", e)
    };

    client_options.app_name = Some("Dodona".to_string());

    let client = match Client::with_options(client_options) {
        Ok(c) => c,
        Err(e) => panic!("Client Creation Failed: {}", e)
    };

    let engine = State {
        client: Arc::new(client)
    };

    let mut app = tide::with_state(engine);

    // Setting up routes
    let mut core_api = app.at("/api");
    core_api.at("/").get(routes::index);
    core_api.at("/hello").get(routes::hello);

    
    // CORS
    let cors = CorsMiddleware::new()
    .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
    .allow_origin(Origin::from("*"))
    .allow_credentials(false);

    app.with(cors);

    // Serving App
    tide::log::start();
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.listen("0.0.0.0:3001").await?;
    Ok(())
}