//! Contains the Distributed Control Layer for the Sybl project.
//!
//! Manages connections to Compute Nodes, a MongoDB database and an Inteface Layer

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
use tokio::sync::mpsc::{self, Receiver, Sender};

<<<<<<< HEAD:backend/dcl/src/lib.rs
=======
pub mod config;
pub mod health;
>>>>>>> health checking functionally there:dcl/src/lib.rs
pub mod interface_end;
pub mod job_end;
pub mod node_end;

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
    let interface_socket =
        u16::from_str(&env::var("INTERFACE_SOCKET").expect("INTERFACE_SOCKET must be set"))
            .unwrap();
    let node_socket =
        u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();
    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);
    let client = Arc::new(
        Client::with_options(client_options)
            .unwrap()
            .database("sybl"),
    );
    let db_conn_interface = client.clone();
    let nodepool = Arc::new(node_end::NodePool::new());
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(20);

    tokio::spawn(async move {
        interface_end::run(interface_socket, db_conn_interface, tx)
            .await
            .unwrap();
    });

    let db_conn_node = client.clone();
    let nodepool_clone = nodepool.clone();
    tokio::spawn(async move {
        node_end::run(nodepool_clone, node_socket, db_conn_node)
            .await
            .unwrap();
    });

    let nodepool_clone = nodepool.clone();
    tokio::spawn(async move {
        job_end::run(nodepool_clone, rx).await.unwrap();
    });

    let nodepool_clone = nodepool.clone();
    tokio::spawn(async move {
        health::health_runner(nodepool_clone, 5).await;
    })
    .await?;

    log::info!("(DCL) shutting down...");

    Ok(())
}
