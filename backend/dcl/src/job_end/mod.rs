//! Part of DCL that takes a DCN and a dataset and comunicates with node

use bytes::Bytes;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::cmp::max;
use std::collections::HashMap;
use std::convert::From;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{Notify, RwLock};
use tokio::time::timeout;

use crate::node_end::NodePool;
use crate::JobControl;
use messages::{ClientMessage, ReadLengthPrefix, WriteLengthPrefix};
use models::gridfs;
use models::jobs::JobConfiguration;
use models::jobs::PredictionType;
use models::predictions::Prediction;
use models::projects::Status;

use utils::anon::{anonymise_dataset, deanonymise_dataset, infer_dataset_columns};
use utils::compress::compress_data;
use utils::finance::reimburse;
use utils::generate_ids;
use utils::{Column, Columns};

pub mod ml;

const PREDICTION_CHUNK_SIZE: usize = 10_000;

/// Struct to pass information for a cluster to function
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    /// Id of project
    pub project_id: ObjectId,
    /// Columns in dataset
    pub columns: Columns,
    /// Config
    pub config: JobConfiguration,
    /// Validation results
    pub validation_ans: HashMap<(ModelID, String), String>,
    /// Test record IDs
    pub prediction_rids: HashMap<(ModelID, String), usize>,
    /// Timeout for the job
    pub timeout: Duration,
}

/// The `String` predictions of model `ModelID` on test example `usize`
pub type ModelPredictions = HashMap<(ModelID, usize), String>;
/// The `f64` error of model `ModelID`, or `None` if the model behaved maliciously
pub type ModelErrors = HashMap<ModelID, Option<f64>>;
/// The `f64` weight of model `ModelID` based on validation accuracy and performance
pub type ModelWeights = HashMap<ModelID, f64>;
/// The `String` predictions based on the order of test examples `usize` given
pub type Predictions = HashMap<usize, String>;

/// Memory which can be written back to from threads for
/// prediction related data
#[derive(Debug, Clone, Default)]
pub struct WriteBackMemory {
    /// HashMap of predictions
    pub predictions: Arc<Mutex<ModelPredictions>>,
    /// HashMap of Errors
    pub errors: Arc<Mutex<ModelErrors>>,
}

impl WriteBackMemory {
    /// Creates new instance of WriteBackMemory
    pub fn new() -> WriteBackMemory {
        Self::default()
    }

    /// Function to write back a hashmap of (index, prediction) tuples
    pub fn write_predictions(&self, id: ModelID, pred_map: HashMap<usize, String>) {
        let mut predictions = self.predictions.lock().unwrap();

        for (index, prediction) in pred_map {
            predictions.insert((id.clone(), index), prediction);
        }
    }

    /// Function to write back error value
    pub fn write_error(&self, id: ModelID, error: Option<f64>) {
        let mut errors = self.errors.lock().unwrap();
        errors.insert(id, error);
    }

    /// Gets cloned version of predictions
    pub fn get_predictions(&self) -> ModelPredictions {
        let predictions = self.predictions.lock().unwrap();
        predictions.clone()
    }

    /// Gets cloned version of errors
    pub fn get_errors(&self) -> ModelErrors {
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

// The proportion of training examples to use as validation examples
const VALIDATION_SPLIT: f64 = 0.2;

// inclusion probability used for Bernoulli sampling
const INCLUSION_PROBABILITY: f64 = 0.95;

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
    job_control: JobControl,
) -> Result<()> {
    log::info!("Job End Running");

    loop {
        let jq_filter = job_control.job_queue.filter(&nodepool.active);
        log::info!("Filtered Jobs: {:?}", &jq_filter);
        if jq_filter.is_empty() {
            log::info!("Nothing in the Job Queue to inspect");
            job_control.notify.notified().await;
            log::info!("Something has changed!");
            continue;
        }

        for index in jq_filter {
            let (project_id, msg, config) = job_control.job_queue.remove(index);

            let data = msg
                .train
                .trim()
                .split('\n')
                .chain(msg.predict.trim().split('\n').skip(1))
                .collect::<Vec<_>>()
                .join("\n");

            let mut columns = infer_dataset_columns(&data).unwrap();

            log::info!("Columns: {:?}", &columns);

            let config_clone = config.clone();
            let anon_config = ClientMessage::from(config_clone);

            let cluster = match nodepool
                .build_cluster(anon_config.anonymise(&columns))
                .await
            {
                Some(c) => c,
                _ => {
                    log::info!("No cluster could be built");

                    job_control
                        .job_queue
                        .insert(index, (project_id, msg, config));

                    continue;
                }
            };

            let mut train = msg
                .train
                .trim()
                .split('\n')
                .enumerate()
                .filter(|(i, _)| *i == 0 || thread_rng().gen::<f64>() < INCLUSION_PROBABILITY)
                .map(|(_, t)| t)
                .collect::<Vec<_>>();

            let headers = train.remove(0);
            let mut validation = Vec::new();
            let test = msg.predict.trim().split('\n').skip(1).collect::<Vec<_>>();

            let JobConfiguration {
                prediction_column,
                prediction_type,
                timeout,
                ..
            } = config.clone();

            if prediction_type == PredictionType::Classification {
                columns.insert(
                    prediction_column.clone(),
                    Column::categorical(&prediction_column, &data),
                );
            }

            for _ in 0..max(1, (train.len() as f64 * VALIDATION_SPLIT) as usize) {
                validation.push(train.swap_remove(thread_rng().gen_range(0..train.len())));
            }
            log::info!("Built validation data");

            let (bags, validation_ans, prediction_rids) = prepare_cluster(
                &cluster,
                headers,
                &train,
                &test,
                &validation,
                &columns,
                &prediction_column,
            );
            let info = ClusterInfo {
                project_id: project_id.clone(),
                columns: columns.clone(),
                config: config.clone(),
                validation_ans: validation_ans.clone(),
                prediction_rids: prediction_rids.clone(),
                timeout: Duration::from_secs((timeout * 60) as u64),
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

        log::info!("Nothing in the JobQueue can be run");
        job_control.notify.notified().await;
        log::info!("Something has changed!");
    }
}

/// Function will take all data for a job and will bag the data to prepare for it
/// being distributed among the models. This will return the data being sent to each
/// model, as well as the rids of each prediction example in the test data to enable
/// validation of results, as well as the validation answers.
pub fn prepare_cluster(
    cluster: &HashMap<String, Arc<RwLock<TcpStream>>>,
    headers: &str,
    train: &[&str],
    test: &[&str],
    validation: &[&str],
    columns: &Columns,
    prediction_column: &str,
) -> (
    HashMap<ModelID, (String, String)>,
    HashMap<(ModelID, String), String>,
    HashMap<(ModelID, String), usize>,
) {
    // The test and train datasets associated for each model
    let mut bags: HashMap<ModelID, (String, String)> = HashMap::new();

    // The validation record ids and answers for each model
    let mut validation_ans: HashMap<(ModelID, String), String> = HashMap::new();

    // The test record ids for each model
    let mut prediction_rids: HashMap<(ModelID, String), usize> = HashMap::new();

    for key in cluster.keys() {
        log::info!("BOOTSTRAPPING");

        // current method resamples the same number of training examples
        let model_train: Vec<_> = train
            .choose_multiple(&mut thread_rng(), train.len())
            .map(|s| s.to_owned())
            .collect();

        // Create new train set with headers
        let mut model_anon_train = vec![headers];
        model_anon_train.extend_from_slice(&model_train);

        // Create new test set with headers
        let mut model_anon_test = vec![headers];
        model_anon_test.extend_from_slice(&test);

        // Create new validation set with headers
        let mut model_anon_valid = vec![headers];
        model_anon_valid.extend_from_slice(&validation);

        // Anonymise train data
        let anon_train = anonymise_dataset(&model_anon_train.join("\n"), &columns).unwrap();
        // Anonymise test data
        let anon_test = anonymise_dataset(&model_anon_test.join("\n"), &columns).unwrap();
        // Anonymise validation data
        let anon_valid = anonymise_dataset(&model_anon_valid.join("\n"), &columns).unwrap();

        // Add record ids to train
        let (anon_train, train_rids) = generate_ids(&anon_train);
        log::info!(
            "IDs: {:?}\nAnonymised Train: {:?}",
            &train_rids,
            &anon_train
        );
        // Add record ids to test
        let (anon_test, test_rids) = generate_ids(&anon_test);

        // Record the index associated with each test record id
        for (i, rid) in test_rids.iter().enumerate() {
            prediction_rids.insert((key.clone(), rid.clone()), i);
        }

        log::info!("IDs: {:?}\nAnonymised Test: {:?}", &test_rids, &anon_test);
        // Add record ids to validation
        let (anon_valid_ans, valid_rids) = generate_ids(&anon_valid);
        log::info!(
            "IDs: {:?}\nAnonymised Valid: {:?}",
            &valid_rids,
            &anon_valid_ans
        );

        let mut anon_valid_ans: Vec<_> = anon_valid_ans.trim().lines().collect();
        let mut anon_valid: Vec<String> = Vec::new();

        anon_valid_ans.remove(0);
        let headers = format!("record_id,{}", headers);
        // Remove validation answers and record them for evaluation
        for (record, id) in anon_valid_ans.iter().zip(valid_rids.iter()) {
            let values: Vec<_> = record.split(',').zip(headers.split(',')).collect();
            let anon_ans = values
                .iter()
                .find_map(|(v, h)| (*h == prediction_column).then(|| *v))
                .unwrap()
                .to_string();

            let ans = columns
                .get(prediction_column)
                .unwrap()
                .deanonymise(anon_ans)
                .unwrap();

            validation_ans.insert((key.clone(), id.to_owned()), ans.to_owned());
            anon_valid.push(
                values
                    .iter()
                    .map(|(v, h)| if *h == prediction_column { "" } else { *v })
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        let mut anon_valid: Vec<&str> = anon_valid.iter().map(|s| s.as_ref()).collect();
        let mut anon_test = anon_test.trim().lines().collect::<Vec<_>>();

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

    (bags, validation_ans, prediction_rids)
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

    for (model_id, dcn_stream) in cluster {
        let np_clone = Arc::clone(&nodepool);
        let database_clone = Arc::clone(&database);
        let info_clone = info.clone();
        let wbm_clone = wbm.clone();
        let cc_clone = cc.clone();
        let train_predict = prediction_bag.get(&model_id).unwrap().clone();

        tokio::spawn(async move {
            let wait = info_clone.timeout;

            let future = dcl_protocol(
                np_clone,
                database_clone,
                model_id,
                dcn_stream,
                info_clone,
                cc_clone,
                train_predict,
                wbm_clone,
            );

            timeout(wait, future).await.unwrap()
        });
    }

    let project_id = info.project_id.clone();

    cc.notify.notified().await;
    log::info!("All Jobs Complete!");

    let (weights, predictions) =
        ml::weight_predictions(&wbm.get_predictions(), &wbm.get_errors(), &info);

    // TODO: reimburse clients based on weights
    log::info!("Model weights: {:?}", weights);

    for (model_id, weight) in &weights {
        let database_clone = Arc::clone(&database);
        reimburse(
            database_clone,
            &ObjectId::with_string(model_id).unwrap(),
            info.config.cost,
            *weight,
        )
        .await?;
    }

    let database_clone = Arc::clone(&database);
    ml::model_performance(
        database_clone,
        weights,
        &info.project_id,
        Some(Arc::clone(&nodepool)),
    )
    .await?;

    let database_clone = Arc::clone(&database);
    let malicious: Vec<ModelID> = wbm
        .get_errors()
        .iter()
        .filter_map(|(k, v)| v.is_none().then(|| k.to_string()))
        .collect();
    ml::penalise(
        database_clone,
        malicious,
        &info.project_id,
        Some(Arc::clone(&nodepool)),
    )
    .await?;

    write_predictions(database.clone(), info.project_id, predictions)
        .await
        .unwrap_or_else(|error| log::error!("Error writing predictions to DB: {}", error));

    change_status(database, project_id, Status::Complete).await?;
    Ok(())
}

/// Function to execute DCL protocol
pub async fn dcl_protocol(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    model_id: String,
    stream: Arc<RwLock<TcpStream>>,
    info: ClusterInfo,
    cluster_control: ClusterControl,
    train_predict: (String, String),
    write_back: WriteBackMemory,
) -> Result<()> {
    log::info!("Sending a job to node with key: {}", &model_id);

    let mut dcn_stream = stream.write().await;

    let mut buffer = [0_u8; 1024];
    let (train, predict) = train_predict;

    let dataset_message = ClientMessage::Dataset { train, predict };

    dcn_stream.write(&dataset_message.as_bytes()).await.unwrap();

    // TODO: Propagate this error forward to the frontend so that it can say a node has failed
    let prediction_message =
        match ClientMessage::from_stream(dcn_stream.deref_mut(), &mut buffer).await {
            Ok(pm) => pm,
            Err(error) => {
                nodepool.update_node_alive(&model_id, false).await;
                cluster_control.decrement().await;

                log::error!(
                    "(Node {}) Error dealing with node predictions: {}",
                    &model_id,
                    error
                );

                return Ok(());
            }
        };

    let raw_anonymised_predictions = match prediction_message {
        ClientMessage::Predictions(s) => s,
        _ => unreachable!(),
    };
    let anonymised_predictions = raw_anonymised_predictions.trim();

    log::info!("Predictions: {:?}", &anonymised_predictions);

    if let Some((model_predictions, model_error)) =
        deanonymise_dataset(&anonymised_predictions, &info.columns)
            .and_then(|predictions| ml::evaluate_model(&model_id, &predictions, &info))
    {
        log::info!(
            "(Node: {}) Computed Data: {:?}",
            &model_id,
            &model_predictions
        );
        write_back.write_error(model_id.clone(), Some(model_error));
        write_back.write_predictions(model_id.clone(), model_predictions);
    } else {
        log::info!("(Node: {}) Failed to respond correctly", &model_id);
        write_back.write_error(model_id.clone(), None);
    }

    increment_run_count(database, &model_id).await?;

    nodepool.end(&model_id).await?;

    cluster_control.decrement().await;

    Ok(())
}

/// Writes predictions back to the Mongo database for long term storage.
///
/// Write predictions back to the database using the GridFS interface. This allows
/// bigger files to be stored for long term storage of results.
pub async fn write_predictions(
    database: Arc<Database>,
    id: ObjectId,
    dataset: Vec<String>,
) -> Result<()> {
    let predictions = database.collection("predictions");

    let mut pred_dataset = gridfs::File::new(String::from("predictions"));

    for chunk in dataset.chunks(PREDICTION_CHUNK_SIZE) {
        let joined = chunk.join("\n");
        let compressed = Bytes::from(compress_data(&joined)?);
        pred_dataset.upload_chunk(&database, &compressed).await?;
    }

    pred_dataset.finalise(&database).await?;

    // Convert to a document and insert it
    let prediction = Prediction::new(id, pred_dataset.id);
    let document = mongodb::bson::ser::to_document(&prediction)?;
    predictions.insert_one(document, None).await?;

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
