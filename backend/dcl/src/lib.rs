//! Contains the Distributed Control Layer for the Sybl project.
//!
//! Manages connections to Compute Nodes, a `MongoDB` database and an Interface Layer

#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

#[macro_use]
extern crate serde;

use anyhow::Result;
use mongodb::options::ClientOptions;
use mongodb::Client;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;

pub mod health;
pub mod interface_end;
pub mod job_end;
pub mod node_end;
pub mod protocol;

/// A pair of datasets, one for training and one for predicting.
#[derive(Debug)]
pub struct DatasetPair {
    /// The training dataset
    pub train: String,
    /// The prediction dataset
    pub predict: String,
}

/// Main runner function for the DCL
///
/// This function is called when starting up the DCL. It starts the
/// tokio runtime and sets up its connection with the MongoDB database.
/// It will then spawn threads for the different parts of the DCL to
/// offer the full functionality of the product.
#[tokio::main]
pub async fn run() -> Result<()> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let broker_socket =
        u16::from_str(&env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string()))
            .expect("BROKER_PORT must be a u16");
    let node_socket =
        u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();

    let health = u64::from_str(&env::var("HEALTH").expect("HEALTH must be set")).unwrap();
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("sybl"));

    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);
    let client = Arc::new(
        Client::with_options(client_options)
            .unwrap()
            .database(&database_name),
    );
    let db_conn_interface = Arc::clone(&client);
    let nodepool = Arc::new(node_end::NodePool::new());
    let (tx, rx) = mpsc::channel(20);

    tokio::spawn(async move {
        interface_end::run(broker_socket, db_conn_interface, tx)
            .await
            .unwrap();
    });

    let nodepool_clone = Arc::clone(&nodepool);
    let node_client = Arc::clone(&client);
    tokio::spawn(async move {
        node_end::run(nodepool_clone, node_client, node_socket)
            .await
            .unwrap();
    });

    let nodepool_clone = Arc::clone(&nodepool);
    let job_client = Arc::clone(&client);
    tokio::spawn(async move {
        job_end::run(nodepool_clone, job_client, rx).await.unwrap();
    });

    let health_client = Arc::clone(&client);
    let nodepool_clone = Arc::clone(&nodepool);
    tokio::spawn(async move {
        health::health_runner(health_client, nodepool_clone, health).await;
    })
    .await?;

    log::info!("(DCL) shutting down...");

    Ok(())
}
