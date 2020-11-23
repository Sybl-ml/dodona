//! Part of DCL that takes a DCN and a dataset and comunicates with node

use anyhow::Result;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use utils::read_stream;

use crate::node_end::NodePool;

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node. The node will then do computation
/// on that dataset and will read in information from comp node.
pub async fn run(
    nodepool: Arc<NodePool>,
    mut rx: Receiver<String>,
    job_timeout: u64,
) -> Result<()> {
    let timeout = Duration::from_secs(job_timeout);
    log::info!("RUNNING JOB END");

    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", &msg);

        let np_clone = Arc::clone(&nodepool);
        tokio::spawn(async move {
            if let Some((key, dcn)) = np_clone.get().await {
                let mut dcn_stream = dcn.write().await;
                // This is temporary, planning on creating seperate place for defining messages
                let check = "{'Dataset': 'here'}";
                dcn_stream.write(check.as_bytes()).await.unwrap();
                let check_res: Vec<u8> = read_stream(&mut dcn_stream, timeout.clone()).await;
                log::info!("Check Result: {}", from_utf8(&check_res).unwrap());
                dcn_stream.write(msg.as_bytes()).await.unwrap();
                let dataset: Vec<u8> = read_stream(&mut dcn_stream, timeout.clone()).await;
                log::info!("Computed Data: {}", from_utf8(&dataset).unwrap());
                np_clone.end(key).await;
            }
        });
    }
    Ok(())
}
