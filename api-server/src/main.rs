use std::env;

use async_std::sync::Arc;
use dotenv::dotenv;
use http_types::headers::HeaderValue;
use mongodb::options::ClientOptions;
use mongodb::Client;
use tide::security::{CorsMiddleware, Origin};

use dodona::routes;
use dodona::State;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");

    // Configuring DB connection
    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();

    let engine = State {
        client: Arc::new(client),
        db_name: Arc::new(String::from("sybl")),
    };

    let mut app = tide::with_state(engine);

    // Setting up routes
    let mut core_api = app.at("/api");
    core_api.at("/").get(routes::index);
    core_api.at("/hello").get(routes::hello);

    let mut user_api = app.at("/api/users");
    user_api.at("/:user_id").get(routes::users::get);
    user_api.at("/filter").post(routes::users::filter);
    user_api.at("/edit").post(routes::users::edit);

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
