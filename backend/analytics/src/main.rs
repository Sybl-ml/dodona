use config::Environment;
use utils::setup_logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger("analytics");

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };
    config::load(environment);

    analytics::run().await?;

    Ok(())
}
