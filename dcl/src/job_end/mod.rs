//! Part of DCL that takes a DCN and a dataset and comunicates with node

use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::node_end::ServerPool;

/// Starts up and runs the job end
///
/// Takes in serverpool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the serverpool and
/// the dataset will be written to that node.
pub async fn run(serverpool: Arc<ServerPool>, mut rx: Receiver<String>) -> Result<()> {
    log::info!("RUNNING JOB END");
    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", msg);
        let (key, dcn): (ObjectId, Arc<RwLock<TcpStream>>) = serverpool.get().await.unwrap();
        let mut dcn_write = dcn.write().await;
        dcn_write.write(msg.as_bytes()).await.unwrap();
        serverpool.end(key).await;
    }
    Ok(())
}
