//! Health checking functionality
//!
//! This will go through and will check each node to make sure
//! each is alive and working. It will update its status in the
//! NodeInfo object for the node.

use crate::node_end::NodePool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::RwLock;

/// Helper struct to send data to a Node to check if it is alive
#[derive(Serialize, Deserialize, Debug)]
struct Health {
    alive: u8,
}

impl Health {
    /// Creates a new Health instance
    pub fn new(alive: u8) -> Health {
        Health { alive }
    }
}

/// Runner for health checking
///
/// Runs the health checking framework to go through
/// each node that is not currently being used and makes
/// sure it is still alive. This will be run every <delay> seconds.
pub async fn health_runner(nodepool: Arc<NodePool>, delay: u64) {
    log::info!("HEALTH CHECKING UP");
    let mut interval = tokio::time::interval(Duration::from_secs(delay));

    loop {
        let np = nodepool.clone();
        check_health(np).await;
        log::info!("HEALTH CHECKED");
        interval.tick().await;
    }
}

/// Go through nodes and check if alive
///
/// Loops through all nodes and checks to see if they are alive.
/// This information is saved in NodeInfo.
pub async fn check_health(nodepool: Arc<NodePool>) {
    let nodes_read = nodepool.nodes.read().await;

    for (oid, node) in nodes_read.iter() {
        if !nodepool.is_using(&oid).await {
            let nd_hb = heartbeat(node.get_tcp()).await;
            nodepool.update_node(nd_hb, &oid).await;
        }
    }
}

/// Checks to see if a Node is still alive
///
/// Checks to see if a node is still alive by sending it a
/// small bit of JSON and it waits for its response. If it fails
/// then it is treated as dead. If not then it is treated as alive.
pub async fn heartbeat(stream: Arc<RwLock<TcpStream>>) -> bool {
    let mut stream_write = stream.write().await;
    let check = Health::new(1);
    stream_write
        .write(serde_json::to_string(&check).unwrap().as_bytes())
        .await
        .unwrap();
    let mut buffer = Vec::new();
    match stream_write.read(&mut buffer).await {
        Ok(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests;
