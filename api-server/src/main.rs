use std::env;
use std::sync::Arc;

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
    projects_api.at("/").get(routes::projects::get_all);
    projects_api
        .at("/p/:project_id")
        .get(routes::projects::get_project);
    projects_api.at("/u/:user_id/new").post(routes::projects::new);
    projects_api.at("/p/:project_id/add").post(routes::projects::add);

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
