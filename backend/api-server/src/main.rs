use config::Environment;

#[actix_rt::main]
async fn main() -> actix_web::Result<()> {
    let filters = vec![
        ("api_server", log::LevelFilter::Debug),
        ("config", log::LevelFilter::Debug),
        ("models", log::LevelFilter::Debug),
        ("actix_web", log::LevelFilter::Debug),
        ("actix_server", log::LevelFilter::Info),
    ];

    utils::setup_logger_with_filters(filters);

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);

    let server = api_server::build_server().await?;
    server.await?;

    Ok(())
}
