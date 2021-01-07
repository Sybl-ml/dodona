//! Contains the API server for the Sybl website.
//!
//! Manages the backend with a Mongo database and responds to frontend requests for data.

#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

#[macro_use]
extern crate serde_json;

use std::env;
use std::str::FromStr;

use mongodb::options::ClientOptions;
use mongodb::Client;

use actix_cors::Cors;
use actix_web::{http, middleware, App, HttpServer, Result};

pub mod dodona_error;
pub mod routes;

/// Defines the state for each request to access.
#[derive(Clone, Debug)]
pub struct AppState {
    /// An instance of the MongoDB client
    pub client: Client,
    /// The name of the database to access
    pub db_name: String,
    /// The pepper to use when hashing
    pub pepper: String,
    /// The number of iterations to use for hashing
    pub pbkdf2_iterations: u32,
}

/// Builds the Tide server.
///
/// Creates a new Tide server instance and adds the API routes to it, along with setting up the
/// [`State`] that each request has access to. This allows the server to be set up externally more
/// easily, by simply building it and then calling the `listen` method.
///
/// # Examples
///
/// ```no_run
/// #[async_std::main]
/// async fn main() -> std::io::Result<()> {
///     let server = api_server::build_server().await;
///     server.listen("localhost:3000").await?;
///
///     Ok(())
/// }
/// ```
pub async fn build_server() -> Result<()> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let pepper = env::var("PEPPER").expect("PEPPER must be set");
    let pbkdf2_iterations = env::var("PBKDF2_ITERATIONS").expect("PBKDF2_ITERATIONS must be set");

    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();

    HttpServer::new(move || {
        // cors
        let cors_middleware = Cors::default()
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        // launch http server
        App::new()
            .wrap(cors_middleware)
            .wrap(middleware::Logger::default())
            // https://github.com/actix/examples/blob/8dab533b40d9d0640e5c75922c9e8e292ed4a7d5/sqlx_todo/src/main.rs#L41
            // pass database pool to application so we can access it inside handlers
            .data(AppState {
                client: client.clone(),
                db_name: String::from("sybl"),
                pepper: pepper.clone(),
                pbkdf2_iterations: u32::from_str(&pbkdf2_iterations)
                    .expect("PBKDF2_ITERATIONS must be parseable as an integer"),
            })
            .configure(routes::init)
    })
    .bind("0.0.0.0:3001")?
    .run()
    .await
    .unwrap();

    Ok(())
}
