use std::env;

use config::Environment;

#[tokio::main]
async fn main() {
    let filters = vec![
        ("dcl", log::LevelFilter::Debug),
        ("config", log::LevelFilter::Debug),
        ("messages", log::LevelFilter::Debug),
        ("models", log::LevelFilter::Debug),
        ("utils", log::LevelFilter::Debug),
    ];

    utils::setup_logger_with_filters(filters);

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);

    // Decide whether to run as the control node or an edge node
    let result = if env::args().find(|arg| arg == "control").is_some() {
        dcl::run_as_controller().await
    } else {
        dcl::run().await
    };

    if let Err(e) = result {
        log::error!("Error occurred: {}", e);
    }
}
