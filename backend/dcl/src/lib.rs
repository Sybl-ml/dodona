//! Contains the Distributed Control Layer for the Sybl project.
//!
//! Manages connections to Compute Nodes, a `MongoDB` database and an Interface Layer

#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

#[macro_use]
extern crate serde;

use anyhow::Result;
use messages::ClientMessage;
use mongodb::bson::oid::ObjectId;
use mongodb::options::ClientOptions;
use mongodb::Client;
use std::collections::VecDeque;
use std::env;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::sync::Notify;

pub mod health;
pub mod interface_end;
pub mod job_end;
pub mod node_end;
pub mod protocol;

/// Struct to hold Job Queue for DCL
#[derive(Debug, Default, Clone)]
pub struct JobQueue(Arc<Mutex<VecDeque<(ObjectId, DatasetPair, ClientMessage)>>>);

impl JobQueue {
    /// Creates a new instance of the [`JobQueue`] struct
    pub fn new() -> Self {
        Self::default()
    }

    /// Goes through jobs in the [`JobQueue`] and returns a vector
    /// containing the indexes of the jobs which the DCL can execute with
    /// its current `active` nodes.
    pub fn filter(&self, active: &AtomicUsize) -> Vec<usize> {
        let jq_mutex = self.0.lock().unwrap();

        let jq_filter: Vec<_> = jq_mutex
            .iter()
            .enumerate()
            .filter(|(_, (_, _, config))| match config {
                ClientMessage::JobConfig { cluster_size, .. } => {
                    (*cluster_size as usize) <= active.load(Ordering::SeqCst)
                }
                _ => false,
            })
            .map(|(idx, _)| idx)
            .collect();

        return jq_filter;
    }

    /// Using an index, this function will remove the required job from the [`JobQueue`]. This is so that
    /// it gives an ownership of the data to the caller of the function.
    pub fn remove(&self, index: usize) -> (ObjectId, DatasetPair, ClientMessage) {
        let mut jq_mutex = self.0.lock().unwrap();

        jq_mutex
            .remove(index)
            .expect("Tried to get value from invalid index")
    }

    /// Puts a job back in the [`JobQueue`] if it is not being executed. This will place it in a location
    /// specified by the index parameter. This will be the place in the [`JobQueue`] that it
    /// previously was.
    pub fn insert(&self, index: usize, job: (ObjectId, DatasetPair, ClientMessage)) {
        let mut jq_mutex = self.0.lock().unwrap();

        jq_mutex.insert(index, job);
    }

    /// Enables a job to be pushed onto the end of the [`JobQueue`] when it
    /// arrives in the DCL.
    pub fn push(&self, job: (ObjectId, DatasetPair, ClientMessage)) {
        let mut job_queue_write = self.0.lock().unwrap();

        job_queue_write.push_back(job);
    }
}

/// A pair of datasets, one for training and one for predicting.
#[derive(Debug)]
pub struct DatasetPair {
    /// The training dataset
    pub train: String,
    /// The prediction dataset
    pub predict: String,
}

/// Data structures for running job control in the DCL
#[derive(Debug, Default, Clone)]
pub struct JobControl {
    /// Job Queue for jobs coming from the interface
    pub job_queue: JobQueue,
    /// Notify struct to improve performance of job end
    pub notify: Arc<Notify>,
}

impl JobControl {
    /// New instance of JobControl
    pub fn new() -> Self {
        Self::default()
    }
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

    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);
    let client = Arc::new(
        Client::with_options(client_options)
            .unwrap()
            .database("sybl"),
    );

    let job_control = JobControl::new();
    let job_notify = Arc::clone(&job_control.notify);
    let nodepool = Arc::new(node_end::NodePool::new(job_notify));

    let db_conn_interface = Arc::clone(&client);
    let jc_clone = job_control.clone();
    tokio::spawn(async move {
        interface_end::run(broker_socket, db_conn_interface, jc_clone)
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
    let jc_clone = job_control.clone();
    tokio::spawn(async move {
        job_end::run(nodepool_clone, job_client, jc_clone)
            .await
            .unwrap();
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
