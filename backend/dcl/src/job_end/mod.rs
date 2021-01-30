//! Part of DCL that takes a DCN and a dataset and comunicates with node

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

use utils::anon::{anonymise_dataset, deanonymise_dataset, infer_dataset_columns};
use utils::compress::compress_bytes;
use utils::generate_ids;
use utils::Columns;

pub mod finance;

/// Struct to pass information for a cluster to function
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    /// Id of project
    pub id: ObjectId,
    /// Columns in dataset
    pub columns: Columns,
    /// Config
    pub config: ClientMessage,
    /// Validation results
    pub validation_ans: HashMap<(ModelID, String), String>,
}

/// Memory which can be written back to from threads for
/// prediction related data
#[derive(Debug, Clone)]
pub struct WriteBackMemory {
    /// HashMap of predictions
    pub predictions: Arc<Mutex<HashMap<(ModelID, String), String>>>,
    /// HashMap of Errors
    pub errors: Arc<Mutex<HashMap<ModelID, f64>>>,
}

impl WriteBackMemory {
    /// Creates new instance of WriteBackMemory
    pub fn new() -> WriteBackMemory {
        WriteBackMemory {
            predictions: Arc::new(Mutex::new(HashMap::new())),
            errors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Function to write back a hashmap of (record_id, prediction) tuples
    pub fn write_predictions(&self, id: ModelID, pred_map: HashMap<String, String>) {
        let mut predictions = self.predictions.lock().unwrap();
        for (record_id, prediction) in pred_map.into_iter() {
            predictions.insert((id.clone(), record_id), prediction);
        }
    }

    /// Function to write back error value
    pub fn write_error(&self, id: ModelID, error: f64) {
        let mut errors = self.errors.lock().unwrap();
        errors.insert(id, error);
    }

    /// Gets cloned version of predictions
    pub fn get_predictions(&self) -> HashMap<(ModelID, String), String> {
        let predictions = self.predictions.lock().unwrap();
        predictions.clone()
    }

    /// Gets cloned version of errors
    pub fn get_errors(&self) -> HashMap<ModelID, f64> {
        let errors = self.errors.lock().unwrap();
        errors.clone()
    }
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

// TODO: Find a better way of identifying models

/// ModelID type
pub type ModelID = String;

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
        let headers = train.remove(0);
        let mut validation = vec![];
        let test = msg.predict.split('\n').skip(1).collect::<Vec<_>>();

        log::info!("{:?}", &train);
        log::info!("{}", &train.len());

        for _ in 1..=VALIDATION_SIZE {
            validation.push(train.swap_remove(thread_rng().gen_range(0..train.len())));
        }

        log::info!("Built validation data");

        // The test and train datasets associated for each model
        let mut bags: HashMap<ModelID, (String, String)> = HashMap::new();

        // The validation record ids and answers for each model
        let mut validation_ans: HashMap<(ModelID, String), String> = HashMap::new();

        loop {
            if let Some(cluster) = nodepool.get_cluster(CLUSTER_SIZE, config.clone()).await {
                log::info!("Created Cluster");

                for (key, _) in &cluster {
                    log::info!("BOOTSTRAPPING");
                    let model_train: Vec<_> = train
                        .choose_multiple(&mut thread_rng(), TRAINING_BAG_SIZE)
                        .map(|s| s.to_owned())
                        .collect();

                    // Create new train set with headers
                    let mut model_anon_train = vec![headers.clone()];
                    model_anon_train.extend_from_slice(&model_train);

                    // Create new test set with headers
                    let mut model_anon_test = vec![headers.clone()];
                    model_anon_test.extend_from_slice(&test);

                    // Create new validation set with headers
                    let mut model_anon_valid = vec![headers.clone()];
                    model_anon_valid.extend_from_slice(&validation);

                    // Anonymise train data
                    let anon_train =
                        anonymise_dataset(&model_anon_train.join("\n"), &columns).unwrap();
                    // Anonymise test data
                    let anon_test =
                        anonymise_dataset(&model_anon_test.join("\n"), &columns).unwrap();
                    // Anonymise validation data
                    let anon_valid =
                        anonymise_dataset(&model_anon_valid.join("\n"), &columns).unwrap();

                    // Add record ids to train
                    let (anon_train, train_rids) = generate_ids(anon_train);
                    log::info!(
                        "IDs: {:?}\nAnonymised Train: {:?}",
                        &train_rids,
                        &anon_train
                    );
                    // Add record ids to test
                    let (anon_test, test_rids) = generate_ids(anon_test);
                    log::info!("IDs: {:?}\nAnonymised Test: {:?}", &test_rids, &anon_test);
                    // Add record ids to validation
                    let (anon_valid_ans, valid_rids) = generate_ids(anon_valid);
                    log::info!(
                        "IDs: {:?}\nAnonymised Valid: {:?}",
                        &valid_rids,
                        &anon_valid_ans
                    );

                    let mut anon_valid_ans: Vec<_> = anon_valid_ans.split("\n").collect();
                    let mut anon_valid: Vec<&str> = vec![];
                    let headers = anon_valid_ans.remove(0);

                    // For now, we assume that the last column is the prediction column
                    let prediction_column = headers.split(',').last().unwrap();

                    // Remove validation answers and record them for evaluation
                    for (record, id) in anon_valid_ans.iter().zip(valid_rids.iter()) {
                        let values: Vec<_> = record.rsplitn(2, ',').collect();
                        let ans = columns
                            .get(prediction_column)
                            .unwrap()
                            .deanonymise(values[0].to_owned())
                            .unwrap();
                        validation_ans.insert((key.clone(), id.to_owned()), ans.to_owned());
                        anon_valid.push(values[1]);
                    }

                    let mut anon_test = anon_test.split("\n").collect::<Vec<_>>();

                    // Get the new anonymised headers for test set
                    let new_headers = anon_test.remove(0);

                    // Combine validation with test
                    anon_test.append(&mut anon_valid);
                    anon_test.shuffle(&mut thread_rng());
                    let mut final_anon_test = vec![new_headers];
                    final_anon_test.extend_from_slice(&anon_test);

                    log::info!("Anonymised Test with Validation: {:?}", &final_anon_test);

                    // Add to bag
                    bags.insert(key.clone(), (anon_train, final_anon_test.join("\n")));
                }

                let info = ClusterInfo {
                    id: id.clone(),
                    columns: columns.clone(),
                    config: config.clone(),
                    validation_ans: validation_ans.clone(),
                };

                let np_clone = Arc::clone(&nodepool);
                let database_clone = Arc::clone(&database);
                run_cluster(
                    np_clone,
                    database_clone,
                    cluster,
                    info.clone(),
                    bags.clone(),
                )
                .await?;
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
    prediction_bag: HashMap<ModelID, (String, String)>,
) -> Result<()> {
    let cc: ClusterControl = ClusterControl::new(cluster.len());
    let wbm: WriteBackMemory = WriteBackMemory::new();

    for (key, dcn) in cluster {
        let np_clone = Arc::clone(&nodepool);
        let database_clone = Arc::clone(&database);
        let info_clone = info.clone();
        let wbm_clone = wbm.clone();
        let cc_clone = cc.clone();
        let train_predict = prediction_bag.get(&key).unwrap().clone();

        tokio::spawn(async move {
            dcl_protcol(
                np_clone,
                database_clone,
                key,
                dcn,
                info_clone,
                cc_clone,
                train_predict,
                wbm_clone,
            )
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
    train_predict: (String, String),
    write_back: WriteBackMemory,
) -> Result<()> {
    log::info!("Sending a job to node with key: {}", key);

    let mut dcn_stream = stream.write().await;

    let mut buffer = [0_u8; 1024];
    let (train, predict) = train_predict;

    let dataset_message = ClientMessage::Dataset {
        train: train,
        predict: predict,
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

    log::info!("Predictions: {:?}", &anonymised_predictions);

    let predictions = deanonymise_dataset(&anonymised_predictions, &info.columns).unwrap();

    // stores the total error penalty for each model
    let mut model_error: f64 = 1.0;
    let mut model_predictions: HashMap<String, String> = HashMap::new();

    // TODO: implement job type recognition through job config struct
    let job_type = "classification";

    for values in predictions
        .split('\n')
        .map(|s| s.split(',').collect::<Vec<_>>())
    {
        let (record_id, prediction) = (values[0].to_owned(), values[1].to_owned());
        let example = (key.clone(), record_id.clone());
        match (info.validation_ans.get(&example), job_type) {
            (Some(answer), "classification") => {
                // if the job is a classification problem, record an error if the predictions do not match
                if prediction != *answer {
                    model_error += 1.0;
                }
            }
            (Some(answer), _) => {
                // if the job is a regression problem, record the L2 error of the prediction
                if let (Ok(p), Ok(a)) = (prediction.parse::<f64>(), answer.parse::<f64>()) {
                    model_error += (p - a).powf(2.0);
                }
            }
            (None, _) => {
                model_predictions.insert(record_id, prediction);
            }
        }
    }

    write_back.write_error(key.clone(), model_error);
    write_back.write_predictions(key.clone(), model_predictions);

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
