//! Health checking functionality
//!
//! This will go through and will check each node to make sure
//! each is alive and working. It will update its status in the
//! NodeInfo object for the node.

use crate::messages::Message;
use crate::node_end::NodePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::RwLock;
use utils::read_stream;

/// Runner for health checking
///
/// Runs the health checking framework to go through
/// each node that is not currently being used and makes
/// sure it is still alive. This will be run every <delay> seconds.
pub async fn health_runner(nodepool: Arc<NodePool>, delay: u64) {
    log::info!("HEALTH CHECKING UP");
    let mut interval = tokio::time::interval(Duration::from_secs(delay));

    loop {
        let np = Arc::clone(&nodepool);
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
    if stream_write
        .write(Message::send(Message::Alive).as_bytes())
        .await
        .is_err()
    {
        return false;
    }

    read_stream(&mut stream_write, Duration::from_millis(100))
        .await
        .is_ok()
}

#[cfg(test)]
mod tests;
