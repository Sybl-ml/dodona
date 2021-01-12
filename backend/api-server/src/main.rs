use config::Environment;
use std::env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };
    config::load(environment);

    api_server::build_server().await.unwrap();

    Ok(())
}
