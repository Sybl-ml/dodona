extern crate tokio;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

use anyhow::Result;

pub mod backend;
pub mod frontend;
pub mod config;
pub mod utils;

#[tokio::main]
pub async fn run() -> Result<()> {
    tokio::spawn(async move {
        frontend::run_server().await.unwrap();
    }).await?;
    
    log::info!("(DCL) shutting down...");

    Ok(())
}

/// Loads the configuration for a given environment into environment variables.
///
/// Given the current environment, loads the configuration file and resolves it based on the given
/// environment, before populating the environment variables with the values contained.
pub fn load_config(environment: config::Environment) {
    let config = config::ConfigFile::from_file("config.toml");
    let resolved = config.resolve(environment);
    resolved.populate_environment();
}
