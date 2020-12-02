use config::Environment;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);
    let app = api_server::build_server().await;

    app.listen("0.0.0.0:3001").await?;

    Ok(())
}
