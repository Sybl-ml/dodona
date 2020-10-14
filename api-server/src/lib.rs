#[macro_use]
extern crate serde_json;

use std::env;
use std::sync::Arc;

use http_types::headers::HeaderValue;
use mongodb::options::ClientOptions;
use mongodb::Client;
use tide::security::{CorsMiddleware, Origin};

pub mod models;
pub mod routes;

#[derive(Clone, Debug)]
pub struct State {
    pub client: Arc<Client>,
    pub db_name: Arc<String>,
    pub pepper: Arc<String>,
}

pub async fn build_server() -> tide::Server<State> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let pepper = env::var("PEPPER").expect("PEPPER must be set");

    // Configuring DB connection
    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();

    let engine = State {
        client: Arc::new(client),
        db_name: Arc::new(String::from("sybl")),
        pepper: Arc::new(pepper),
    };

    let mut app = tide::with_state(engine);

    // Setting up routes
    let mut core_api = app.at("/api");
    core_api.at("/").get(routes::index);

    let mut user_api = app.at("/api/users");
    user_api.at("/:user_id").get(routes::users::get);
    user_api.at("/filter").post(routes::users::filter);
    user_api.at("/edit").post(routes::users::edit);
    user_api.at("/login").post(routes::users::login);
    user_api.at("/new").post(routes::users::new);
    user_api.at("/delete").post(routes::users::delete);

    // CORS
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    // Serving App
    app.at("/").get(|_| async { Ok("Hello, world!") });

    app
}
