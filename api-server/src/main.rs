use dodona::config::Environment;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    dodona::load_config(environment);
    let app = dodona::build_server().await;

<<<<<<< HEAD
=======
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
    projects_api.at("/new").post(routes::projects::new);
    projects_api.at("/add").post(routes::projects::add);

    // CORS
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    // Serving App
>>>>>>> added new endpoint
    tide::log::start();
    app.listen("0.0.0.0:3001").await?;

    Ok(())
}
