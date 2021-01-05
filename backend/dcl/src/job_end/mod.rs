//! Part of DCL that takes a DCN and a dataset and comunicates with node

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::node_end::NodePool;
use crate::DatasetPair;
use messages::client::ClientMessage;
use models::predictions::Prediction;

use utils::anon::{anonymise_dataset, deanonymise_dataset};
use utils::compress::compress_bytes;
use utils::{infer_train_and_predict, Columns};

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
    log::info!("Job End Running");

    while let Some((id, msg)) = rx.recv().await {
        log::info!("Train: {}", &msg.train);
        log::info!("Predict: {}", &msg.predict);

        let data = msg
            .train
            .split("\n")
            .chain(msg.predict.split("\n").skip(1))
            .collect::<Vec<_>>()
            .join("\n");
        let (anon, columns) = anonymise_dataset(data).unwrap();
        let (anon_train, anon_predict) = infer_train_and_predict(&anon);
        let (anon_train_csv, anon_predict_csv) = (anon_train.join("\n"), anon_predict.join("\n"));

        let cluster = nodepool.get_cluster(1).await.unwrap();

        for (key, dcn) in cluster {
            let np_clone = Arc::clone(&nodepool);
            let database_clone = Arc::clone(&database);

            let identifier = id.clone();
            let train = anon_train_csv.clone();
            let predict = anon_predict_csv.clone();
            let cols = columns.clone();

            tokio::spawn(async move {
                dcl_protcol(
                    np_clone,
                    database_clone,
                    key,
                    dcn,
                    identifier,
                    train,
                    predict,
                    cols,
                )
                .await
                .unwrap();
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
    columns: Columns,
) -> Result<()> {
    log::info!("Sending a job to node with key: {}", key);

    let mut dcn_stream = stream.write().await;

    let mut buffer = [0_u8; 1024];

    // This is temporary, planning on creating seperate place for defining messages

    let config = ClientMessage::JobConfig { config: "".into() }.as_bytes();
    dcn_stream.write(&config).await.unwrap();

    let size = dcn_stream.read(&mut buffer).await.unwrap();
    let config_response = std::str::from_utf8(&buffer[..size]).unwrap();

    log::info!("(Node {}) Config response: {}", &key, config_response);

    let dataset_message = ClientMessage::Dataset { train, predict };
    dcn_stream.write(&dataset_message.as_bytes()).await.unwrap();

    // TODO: Propagate this error forward to the frontend so that it can say a node has failed
    let prediction_message = match ClientMessage::from_stream(&mut dcn_stream, &mut buffer).await {
        Ok(pm) => pm,
        Err(error) => {
            nodepool.update_node(&key, false).await?;

            log::error!(
                "(Node {}) Error dealing with node predictions: {}",
                &key,
                error
            );

            return Ok(());
        }
    };

    let anonymised_predictions = match prediction_message {
        ClientMessage::Predictions(s) => s,
        _ => unreachable!(),
    };

    let predictions = deanonymise_dataset(anonymised_predictions, columns).unwrap();

    // Write the predictions back to the database
    write_predictions(database, id, &key, predictions.as_bytes())
        .await
        .unwrap_or_else(|error| {
            log::error!(
                "(Node: {}) Error writing predictions to DB: {}",
                &key,
                error
            )
        });

    log::info!("(Node: {}) Computed Data: {}", &key, predictions);

    nodepool.end(&key).await?;

    Ok(())
}

/// Writes predictions back to the Mongo database for long term storage.
pub async fn write_predictions(
    database: Arc<Database>,
    id: ObjectId,
    model_id: &str,
    dataset: &[u8],
) -> Result<()> {
    let predictions = database.collection("predictions");

    // Compress the data and make a new struct instance
    let compressed = compress_bytes(dataset)?;
    let prediction = Prediction::new(id, compressed);

    // Convert to a document and insert it
    let document = mongodb::bson::ser::to_document(&prediction)?;
    predictions.insert_one(document, None).await?;

    increment_run_count(database, model_id).await?;

    Ok(())
}

/// Increments the run count for the given model in the database.
pub async fn increment_run_count(database: Arc<Database>, model_id: &str) -> Result<()> {
    let models = database.collection("models");
    let object_id = ObjectId::with_string(model_id)?;

    let query = doc! {"_id": &object_id};
    let update = doc! { "$inc": { "times_run": 1 } };
    models.update_one(query, update, None).await?;

    Ok(())
}
