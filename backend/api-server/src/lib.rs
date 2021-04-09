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

use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use actix::prelude::Addr;
use actix::prelude::Recipient;
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer, Result};
use mongodb::{options::ClientOptions, Client, Database};

pub mod auth;
pub mod error;
pub mod routes;

/// Defines the state for each request to access.
#[derive(Clone, Debug)]
pub struct State {
    /// An instance of the MongoDB database
    pub database: Arc<Database>,
    /// The pepper to use when hashing
    pub pepper: Arc<String>,
    /// The number of iterations to use for hashing
    pub pbkdf2_iterations: u32,
}

#[derive(Clone, Debug, Default)]
pub struct WebsocketState {
    /// Map of userids to open sockets
    pub map: Arc<Mutex<HashMap<String, Addr<routes::websockets::ProjectUpdateWs>>>>,
}

#[derive(Clone, Debug, Default)]
pub struct WebsocketState {
    /// Map of userids to open sockets
    pub map: Arc<Mutex<HashMap<String, Addr<routes::websockets::ProjectUpdateWs>>>>,
}

/// Builds the default logging middleware for request logging.
fn build_logging_middleware() -> middleware::Logger {
    middleware::Logger::new("%s @ %r")
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
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("sybl"));

    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();
    let database = Arc::new(client.database(&database_name));

    let map = HashMap::new();
    let shared_state = Arc::new(Mutex::new(map));
    let consumer_state = Arc::clone(&shared_state);

    let websocket_state_data = web::Data::new(WebsocketState { map: shared_state });

    tokio::spawn(async move {
        routes::websockets::consume_updates(9092, consumer_state).await;
    });

    let server = HttpServer::new(move || {
        // cors
        let cors_middleware = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .disable_vary_header()
            .max_age(3600);

        // launch http server
        App::new()
            .wrap(cors_middleware)
            .wrap(build_logging_middleware())
            .data(State {
                database: Arc::clone(&database),
                pepper: Arc::new(pepper.clone()),
                pbkdf2_iterations: u32::from_str(&pbkdf2_iterations)
                    .expect("PBKDF2_ITERATIONS must be parseable as an integer"),
            })
            .app_data(websocket_state_data.clone())
            .route(
                "/api/projects/{project_id}",
                web::get().to(routes::projects::get_project),
            )
            .route(
                "/api/projects/{project_id}",
                web::patch().to(routes::projects::patch_project),
            )
            .route(
                "/api/projects/{project_id}",
                web::delete().to(routes::projects::delete_project),
            )
            .route(
                "/api/projects",
                web::get().to(routes::projects::get_user_projects),
            )
            .route("/api/projects/new", web::post().to(routes::projects::new))
            .route(
                "/api/projects/{project_id}/upload_and_split",
                web::put().to(routes::projects::upload_and_split),
            )
            .route(
                "/api/projects/{project_id}/upload_train_and_predict",
                web::put().to(routes::projects::upload_train_and_predict),
            )
            .route(
                "/api/projects/{project_id}/overview",
                web::post().to(routes::projects::overview),
            )
            .route(
                "/api/projects/{project_id}/data/{dataset_type}",
                web::get().to(routes::projects::get_dataset),
            )
            .route(
                "/api/projects/{project_id}/data",
                web::delete().to(routes::projects::remove_data),
            )
            .route(
                "/api/projects/{project_id}/pagination/{dataset_type}",
                web::get().to(routes::projects::pagination),
            )
            .route(
                "/api/projects/{project_id}/process",
                web::post().to(routes::projects::begin_processing),
            )
            // Clients
            .route(
                "/api/clients/register",
                web::post().to(routes::clients::register),
            )
            .route(
                "/api/clients/models/new",
                web::post().to(routes::clients::new_model),
            )
            .route(
                "/api/clients/models/verify",
                web::post().to(routes::clients::verify_challenge),
            )
            .route(
                "/api/clients/models/{model_id}/unlock",
                web::post().to(routes::clients::unlock_model),
            )
            .route(
                "/api/clients/models/{model_id}/authenticate",
                web::post().to(routes::clients::authenticate_model),
            )
            .route(
                "/api/clients/models",
                web::get().to(routes::clients::get_user_models),
            )
            .route(
                "/api/clients/models/{model_id}/performance",
                web::get().to(routes::clients::get_model_performance),
            )
            // users
            .route("/api/users", web::get().to(routes::users::get))
            .route("/api/users/filter", web::post().to(routes::users::filter))
            .route("/api/users/new", web::post().to(routes::users::new))
            .route(
                "/api/users/avatar",
                web::post().to(routes::users::new_avatar),
            )
            .route(
                "/api/users/avatar",
                web::get().to(routes::users::get_avatar),
            )
            .route("/api/users/edit", web::post().to(routes::users::edit))
            .route("/api/users/login", web::post().to(routes::users::login))
            .route("/api/users/delete", web::post().to(routes::users::delete))
            .service(
                web::resource("/project_updates").route(web::get().to(routes::websockets::index)),
            )
    })
    .bind("0.0.0.0:3001")?
    .run();

    Ok(server)
}
