extern crate tokio;
#[macro_use]
extern crate serde;

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::Result;
use mongodb::options::ClientOptions;
use mongodb::Client;

pub mod interface_end;
pub mod node_end;
pub mod job_end;
pub mod config;
pub mod utils;
pub mod models;

#[tokio::main]
pub async fn run() -> Result<()> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let interface_socket = u16::from_str(&env::var("INTERFACE_SOCKET").expect("INTERFACE_SOCKET must be set")).unwrap();
    let node_socket = u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();
    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);
    let client = Arc::new(Client::with_options(client_options).unwrap().database("sybl"));
    let db_conn_interface = client.clone();
    let serverpool = Arc::new(node_end::ServerPool::new());

    tokio::spawn(async move {
        interface_end::run_server(interface_socket, db_conn_interface).await.unwrap();
    }).await?;

    let db_conn_node = client.clone();
    let serverpool_clone = serverpool.clone();
    tokio::spawn(async move {
        node_end::run_server(serverpool_clone, node_socket, db_conn_node).await.unwrap();
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
