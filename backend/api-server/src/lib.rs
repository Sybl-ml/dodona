//! Contains the API server for the Sybl website.
//!
//! Manages the backend with a Mongo database and responds to frontend requests for data.

#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::env;
use std::str::FromStr;
use std::sync::Arc;

use mongodb::options::ClientOptions;
use mongodb::Client;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer, Result};

pub mod auth;
pub mod dodona_error;
pub mod routes;

/// Defines the state for each request to access.
#[derive(Clone, Debug)]
pub struct AppState {
    /// An instance of the MongoDB client
    pub client: Arc<Client>,
    /// The name of the database to access
    pub db_name: Arc<String>,
    /// The pepper to use when hashing
    pub pepper: Arc<String>,
    /// The number of iterations to use for hashing
    pub pbkdf2_iterations: u32,
}

/// Builds the `actix-web` server.
///
/// Creates a new `actix-web` server instance and adds the API routes to it, along with setting up
/// the [`State`] that each request has access to. This allows the server to be set up externally
/// more easily, by simply building it and then calling the `listen` method.
///
/// # Examples
///
/// ```no_run
/// #[actix_rt::main]
/// async fn main() -> std::io::Result<()>  {
///     api_server::build_server().await.unwrap();
///
///     Ok(())
/// }
/// ```
pub async fn build_server() -> Result<actix_web::dev::Server> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let pepper = env::var("PEPPER").expect("PEPPER must be set");
    let pbkdf2_iterations = env::var("PBKDF2_ITERATIONS").expect("PBKDF2_ITERATIONS must be set");

    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();

    let server = HttpServer::new(move || {
        // cors
        let cors_middleware = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        // launch http server
        App::new()
            .wrap(cors_middleware)
            .wrap(middleware::Logger::default())
            .data(AppState {
                client: Arc::new(client.clone()),
                db_name: Arc::new(String::from("sybl")),
                pepper: Arc::new(pepper.clone()),
                pbkdf2_iterations: u32::from_str(&pbkdf2_iterations)
                    .expect("PBKDF2_ITERATIONS must be parseable as an integer"),
            })
            .route(
                "/api/projects/p/{project_id}",
                web::get().to(routes::projects::get_project),
            )
            .route(
                "/api/projects/p/{project_id}",
                web::patch().to(routes::projects::patch_project),
            )
            .route(
                "/api/projects/p/{project_id}",
                web::delete().to(routes::projects::delete_project),
            )
            .route(
                "/api/projects",
                web::get().to(routes::projects::get_user_projects),
            )
            .route("/api/projects/new", web::post().to(routes::projects::new))
            .route(
                "/api/projects/p/{project_id}/data",
                web::put().to(routes::projects::add_data),
            )
            .route(
                "/api/projects/p/{project_id}/overview",
                web::post().to(routes::projects::overview),
            )
            .route(
                "/api/projects/p/{project_id}/data",
                web::get().to(routes::projects::get_data),
            )
            .route(
                "/api/projects/p/{project_id}/data",
                web::delete().to(routes::projects::remove_data),
            )
            .route(
                "/api/projects/p/{project_id}/process",
                web::post().to(routes::projects::begin_processing),
            )
            .route(
                "/api/projects/p/{project_id}/predictions",
                web::get().to(routes::projects::get_predictions),
            )
            // Clients
            .route(
                "/api/clients/register",
                web::post().to(routes::clients::register),
            )
            .route(
                "/api/clients/m/new",
                web::post().to(routes::clients::new_model),
            )
            .route(
                "/api/clients/m/verify",
                web::post().to(routes::clients::verify_challenge),
            )
            .route(
                "/api/clients/m/unlock",
                web::post().to(routes::clients::unlock_model),
            )
            .route(
                "/api/clients/m/authenticate",
                web::post().to(routes::clients::authenticate_model),
            )
            .route(
                "/api/clients",
                web::get().to(routes::clients::get_user_models),
            )
            .route(
                "/api/clients/m/performance",
                web::post().to(routes::clients::get_model_performance),
            )
            // users
            .route("/api/users", web::get().to(routes::users::get))
            .route("/api/users/filter", web::post().to(routes::users::filter))
            .route("/api/users/new", web::post().to(routes::users::new))
            .route("/api/users/edit", web::post().to(routes::users::edit))
            .route("/api/users/login", web::post().to(routes::users::login))
            .route("/api/users/delete", web::post().to(routes::users::delete))
    })
    .bind("0.0.0.0:3001")?
    .run();

    Ok(server)
}
