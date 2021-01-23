//! Part of DCL that takes a DCN and a dataset and comunicates with node

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{mpsc::Receiver, Notify, RwLock};

use crate::node_end::NodePool;
use crate::DatasetPair;
use messages::{ClientMessage, ReadLengthPrefix, WriteLengthPrefix};
use models::predictions::Prediction;
use models::projects::Status;

use crypto::generate_string;
use utils::anon::{anonymise_dataset, deanonymise_dataset, infer_dataset_columns};
use utils::compress::compress_bytes;
use utils::{infer_train_and_predict, Columns};

/// Struct to pass information for a cluster to function
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    /// Id of project
    pub id: ObjectId,
    /// Columns in dataset
    pub columns: Columns,
    /// Training CSV
    pub train: String,
    /// Prediction CSV
    pub predict: String,
    /// Config
    pub config: ClientMessage,
}

/// Controlling structures for clusters
#[derive(Debug, Clone)]
pub struct ClusterControl {
    /// Cluster counter
    pub counter: Arc<RwLock<usize>>,
    /// Cluster notifier
    pub notify: Arc<Notify>,
}

impl ClusterControl {
    /// Creates a new instance of ClusterControl
    pub fn new(counter: usize) -> ClusterControl {
        ClusterControl {
            counter: Arc::new(RwLock::new(counter)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Decrements the cluster counter
    pub async fn decrement(&self) {
        let mut write_cc = self.counter.write().await;
        *write_cc -= 1;
        if *write_cc == 0 {
            self.notify.notify_one();
        }
    }
}

const CLUSTER_SIZE: usize = 1;
const VALIDATION_SIZE: usize = 10;
const TRAINING_BAG_SIZE: usize = 10;

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node. The node will then do computation
/// on that dataset and will read in information from comp node.
pub async fn run(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    mut rx: Receiver<(ObjectId, DatasetPair, ClientMessage)>,
) -> Result<()> {
    log::info!("Job End Running");

    while let Some((id, msg, config)) = rx.recv().await {
        log::info!("Train: {}", &msg.train);
        log::info!("Predict: {}", &msg.predict);
        log::info!("Job Config: {:?}", &config);

        let data = msg
            .train
            .split('\n')
            .chain(msg.predict.split('\n').skip(1))
            .collect::<Vec<_>>()
            .join("\n");

        let columns = infer_dataset_columns(&data).unwrap();

        let mut train = msg.train.split('\n').collect::<Vec<_>>();
        let mut validation = vec![];
        let test = msg.train.split('\n').collect::<Vec<_>>();

        for _ in 1..=VALIDATION_SIZE {
            validation.push(train.swap_remove(thread_rng().gen_range(0..train.len())));
        }

        let mut bags: HashMap<usize, (String, String)> = HashMap::new();

        for m in 1..=CLUSTER_SIZE {
            let model_train: Vec<_> = train
                .choose_multiple(&mut thread_rng(), TRAINING_BAG_SIZE)
                .map(|s| s.to_owned())
                .collect();
            let mut model_test: Vec<_> = test.clone();
            model_test.append(&mut validation);
            model_test.shuffle(&mut thread_rng());
            bags.insert(
                m,
                (
                    anonymise_dataset(&model_train.join("\n"), &columns).unwrap(),
                    anonymise_dataset(&model_test.join("\n"), &columns).unwrap(),
                ),
            );
        }

        let anon = anonymise_dataset(&data, &columns).unwrap();
        let (anon_train, anon_predict) = infer_train_and_predict(&anon);
        let (anon_train_csv, anon_predict_csv) = (anon_train.join("\n"), anon_predict.join("\n"));

        let info = ClusterInfo {
            id: id.clone(),
            columns: columns.clone(),
            train: anon_train_csv.clone(),
            predict: anon_predict_csv.clone(),
            config: config.clone(),
        };

        loop {
            if let Some(cluster) = nodepool.get_cluster(1, info.config.clone()).await {
                log::info!("Created Cluster");

                let np_clone = Arc::clone(&nodepool);
                let database_clone = Arc::clone(&database);
                run_cluster(np_clone, database_clone, cluster, info.clone()).await?;
                break;
            }
        }
    }
    Ok(())
}

async fn run_cluster(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    cluster: HashMap<String, Arc<RwLock<TcpStream>>>,
    info: ClusterInfo,
) -> Result<()> {
    let cc: ClusterControl = ClusterControl::new(cluster.len());

    for (key, dcn) in cluster {
        let np_clone = Arc::clone(&nodepool);
        let database_clone = Arc::clone(&database);
        let info_clone = info.clone();
        let cc_clone = cc.clone();

        tokio::spawn(async move {
            dcl_protcol(np_clone, database_clone, key, dcn, info_clone, cc_clone)
                .await
                .unwrap();
        });
    }

    let project_id = info.id.clone();

    cc.notify.notified().await;
    log::info!("All Jobs Complete!");
    change_status(database, project_id, Status::Complete).await?;
    Ok(())
}

/// Function to execute DCL protocol
pub async fn dcl_protcol(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    key: String,
    stream: Arc<RwLock<TcpStream>>,
    info: ClusterInfo,
    cluster_control: ClusterControl,
) -> Result<()> {
    log::info!("Sending a job to node with key: {}", key);

    let mut dcn_stream = stream.write().await;

    let mut buffer = [0_u8; 1024];

    let dataset_message = ClientMessage::Dataset {
        train: info.train,
        predict: info.predict,
    };
    dcn_stream.write(&dataset_message.as_bytes()).await.unwrap();

    // TODO: Propagate this error forward to the frontend so that it can say a node has failed
    let prediction_message = match ClientMessage::from_stream(&mut dcn_stream, &mut buffer).await {
        Ok(pm) => pm,
        Err(error) => {
            nodepool.update_node(&key, false).await?;
            cluster_control.decrement().await;

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

    let predictions = deanonymise_dataset(&anonymised_predictions, &info.columns).unwrap();

    // Write the predictions back to the database
    write_predictions(database, info.id, &key, predictions.as_bytes())
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

    cluster_control.decrement().await;

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

/// Change the status of a project after it has been completed
pub async fn change_status(
    database: Arc<Database>,
    project_id: ObjectId,
    status: Status,
) -> Result<()> {
    let projects = database.collection("projects");

    projects
        .update_one(
            doc! { "_id": &project_id},
            doc! {"$set": {"status": status}},
            None,
        )
        .await?;

    Ok(())
}
