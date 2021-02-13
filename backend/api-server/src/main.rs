use config::Environment;
use std::env;

#[actix_rt::main]
async fn main() -> actix_web::Result<()> {
    env::set_var("RUST_LOG", "debug,actix_web=debug,actix_server=info");
    env_logger::init();

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
