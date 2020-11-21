//! Part of DCL that takes a DCN and a dataset and comunicates with node

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::time::Instant;

use crate::node_end::NodePool;

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node. The node will then do computation
/// on that dataset and will read in information from comp node.
pub async fn run(nodepool: Arc<NodePool>, mut rx: Receiver<String>) -> Result<()> {
    let timeout = Duration::from_secs(600);
    let start = Instant::now();
    log::info!("RUNNING JOB END");
    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", &msg);
        let np_clone = Arc::clone(&nodepool);
        tokio::spawn(async move {
            if let Some((key, dcn)) = np_clone.get().await {
                let mut dcn_stream = dcn.write().await;
                let (dcn_read, mut dcn_write) = dcn_stream.split();
                dcn_write.write(msg.as_bytes()).await.unwrap();
                let mut dcn_read = BufReader::new(dcn_read);
                let mut dataset: String = String::new();
                loop {
                    let result = dcn_read.read_line(&mut dataset).await;
                    match result {
                        Ok(0) => break,
                        Ok(n) => {
                            log::info!("Received {} bytes", n);
                        }
                        _ => {
                            if Instant::now().duration_since(start) <= timeout {
                                tokio::time::sleep(Duration::new(20, 0)).await;
                            } else {
                                break;
                            }
                        }
                    }
                }
                log::info!("Received Data: {}", &dataset);
                np_clone.end(key).await;
            }
        });
    }
    Ok(())
}
