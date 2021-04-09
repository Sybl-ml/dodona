use config::Environment;

fn main() {
    let filters = vec![
        ("dcl", log::LevelFilter::Debug),
        ("config", log::LevelFilter::Debug),
        ("models", log::LevelFilter::Debug),
    ];

    utils::setup_logger_with_filters(filters);

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);

    if let Err(e) = dcl::run() {
        log::error!("Error occurred: {}", e);
    }
}
