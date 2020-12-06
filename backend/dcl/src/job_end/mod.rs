//! Part of DCL that takes a DCN and a dataset and comunicates with node

use anyhow::Result;

use mongodb::{bson::oid::ObjectId, Database};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::messages::Message;
use crate::node_end::NodePool;
use crate::DatasetPair;
use models::predictions::Prediction;

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node. The node will then do computation
/// on that dataset and will read in information from comp node.
pub async fn run(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    mut rx: Receiver<(ObjectId, DatasetPair)>,
) -> Result<()> {
    log::info!("RUNNING JOB END");

    while let Some((id, msg)) = rx.recv().await {
        log::info!("Train: {}", &msg.train);
        log::info!("Predict: {}", &msg.predict);

        let cluster = nodepool.get_cluster(1).await.unwrap();

        for (key, dcn) in cluster {
            let np_clone = Arc::clone(&nodepool);
            let database_clone = Arc::clone(&database);

            let identifier = id.clone();
            let train = msg.train.clone();
            let predict = msg.predict.clone();

            tokio::spawn(async move {
                dcl_protcol(
                    np_clone,
                    database_clone,
                    key,
                    dcn,
                    identifier,
                    train,
                    predict,
                )
                .await;
            });
        }
    }
    Ok(())
}

/// Function to execute DCL protocol
pub async fn dcl_protcol(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    key: String,
    stream: Arc<RwLock<TcpStream>>,
    id: ObjectId,
    train: String,
    predict: String,
) -> String {
    log::info!("Sending a job to node with key: {}", key);

    let mut dcn_stream = stream.write().await;

    let mut buffer = [0_u8; 1024];

    // This is temporary, planning on creating seperate place for defining messages

    let config = Message::JobConfig { config: "".into() }.as_bytes();
    dcn_stream.write(&config).await.unwrap();

    let size = dcn_stream.read(&mut buffer).await.unwrap();
    let config_response = std::str::from_utf8(&buffer[..size]).unwrap();

    log::info!("Config response: {}", config_response);

    let dataset_message = Message::Dataset { train, predict };
    dcn_stream.write(&dataset_message.as_bytes()).await.unwrap();

    let prediction_message = Message::from_stream(&mut dcn_stream, &mut buffer)
        .await
        .unwrap();

    let predictions = match prediction_message {
        Message::Predictions(s) => s,
        _ => unreachable!(),
    };

    // Write the predictions back to the database
    write_predictions(database, id, predictions.as_bytes())
        .await
        .unwrap();

    log::info!("Computed Data: {}", predictions);

    nodepool.end(&key).await;

    predictions
}

/// Writes predictions back to the Mongo database for long term storage.
pub async fn write_predictions(
    database: Arc<Database>,
    id: ObjectId,
    dataset: &[u8],
) -> Result<()> {
    let predictions = database.collection("predictions");

    // Compress the data and make a new struct instance
    let compressed = utils::compress_bytes(dataset)?;
    let prediction = Prediction::new(id, compressed);

    // Convert to a document and insert it
    let document = mongodb::bson::ser::to_document(&prediction)?;
    predictions.insert_one(document, None).await?;

    Ok(())
}
