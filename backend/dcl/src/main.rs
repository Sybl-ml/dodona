use config::Environment;
use dcl::run;

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);

    // Do config stuff here
    let code = {
        if let Err(e) = run() {
            log::error!("ERROR: {}", e);
            1
        } else {
            0
        }
    };
    ::std::process::exit(code);
}
