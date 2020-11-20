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
use std::sync::Arc;

use mongodb::options::ClientOptions;
use mongodb::Client;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};

pub mod routes;

/// Defines the state for each request to access.
#[derive(Clone, Debug)]
pub struct State {
    /// An instance of the MongoDB client
    pub client: Arc<Client>,
    /// The name of the database to access
    pub db_name: Arc<String>,
    /// The pepper to use when hashing
    pub pepper: Arc<String>,
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
pub async fn build_server() -> tide::Server<State> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let pepper = env::var("PEPPER").expect("PEPPER must be set");
    let pbkdf2_iterations = env::var("PBKDF2_ITERATIONS").expect("PBKDF2_ITERATIONS must be set");

    // Configuring DB connection
    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();

    let engine = State {
        client: Arc::new(client),
        db_name: Arc::new(String::from("sybl")),
        pepper: Arc::new(pepper),
        pbkdf2_iterations: u32::from_str(&pbkdf2_iterations)
            .expect("PBKDF2_ITERATIONS must be parseable as an integer"),
    };

    let mut app = tide::with_state(engine);

    // Setting up routes
    let mut core_api = app.at("/api");

    let mut user_api = core_api.at("/users");
    user_api.at("/:user_id").get(routes::users::get);
    user_api.at("/filter").post(routes::users::filter);
    user_api.at("/edit").post(routes::users::edit);
    user_api.at("/login").post(routes::users::login);
    user_api.at("/new").post(routes::users::new);
    user_api.at("/delete").post(routes::users::delete);

    let mut projects_api = core_api.at("/projects");
    projects_api
        .at("/u/:user_id")
        .get(routes::projects::get_user_projects);
    projects_api
        .at("/p/:project_id")
        .get(routes::projects::get_project)
        .patch(routes::projects::patch_project)
        .delete(routes::projects::delete_project);
    projects_api
        .at("/u/:user_id/new")
        .post(routes::projects::new);
    projects_api
        .at("/p/:project_id/data")
        .put(routes::projects::add_data)
        .get(routes::projects::get_data);
    projects_api
        .at("/p/:project_id/overview")
        .post(routes::projects::overview);
    projects_api
        .at("/p/:project_id/process")
        .post(routes::projects::begin_processing);

    let mut client_api = core_api.at("/clients");
    client_api.at("/register").post(routes::clients::register);
    client_api.at("/m/new").post(routes::clients::new_model);

    // CORS
    let headers = "GET, POST, PUT, PATCH, DELETE, OPTIONS"
        .parse::<HeaderValue>()
        .unwrap();
    let cors = CorsMiddleware::new()
        .allow_methods(headers)
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    app
}
