//! Part of DCL that takes a DCN and a dataset and comunicates with node

use anyhow::Result;
use std::sync::Arc;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;

use crate::node_end::NodePool;

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node.
pub async fn run(nodepool: Arc<NodePool>, mut rx: Receiver<String>) -> Result<()> {
    log::info!("RUNNING JOB END");
    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", &msg);
        let np_clone = Arc::clone(&nodepool);
        tokio::spawn(async move {
            while let Some((key, dcn)) = np_clone.get().await {
                let mut dcn_write = dcn.write().await;
                dcn_write.write(msg.as_bytes()).await.unwrap();
                np_clone.end(key).await;
                break;
            }
        });
    }
    Ok(())
}
