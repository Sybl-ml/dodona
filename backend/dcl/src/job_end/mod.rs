//! Part of DCL that takes a DCN and a dataset and comunicates with node

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{atomic::Ordering, Arc, Mutex};

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{Notify, RwLock};

use crate::node_end::NodePool;
use crate::{DatasetPair, JobQueue};
use messages::{ClientMessage, ReadLengthPrefix, WriteLengthPrefix};
use models::jobs::PredictionType;
use models::predictions::Prediction;
use models::projects::Status;

use utils::anon::{anonymise_dataset, deanonymise_dataset, infer_dataset_columns};
use utils::compress::compress_bytes;
use utils::generate_ids;
use utils::{Column, Columns};

pub mod finance;
pub mod ml;


/// Struct to pass information for a cluster to function
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    /// Id of project
    pub project_id: ObjectId,
    /// Columns in dataset
    pub columns: Columns,
    /// Config
    pub config: ClientMessage,
    /// Validation results
    pub validation_ans: HashMap<(ModelID, String), String>,
    /// Test record IDs
    pub prediction_rids: HashMap<(ModelID, String), usize>,
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
        for (index, prediction) in pred_map.into_iter() {
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

const VALIDATION_SIZE: usize = 10;
const TRAINING_BAG_SIZE: usize = 10;

// inclusion probability used for Bernoulli sampling
const INCLUSION_PROBABILITY: f64 = 0.95;

/// ModelID type
pub type ModelID = String;

/// Goes through the jobs in the job queue and only selects ones which have a 
/// cluster size which can be run by the DCL.
pub fn filter_jobs(job_queue: &JobQueue, nodepool: &Arc<NodePool>) -> Vec<usize>{
    let jq_mutex = job_queue
    .lock()
    .unwrap();

    let jq_filter: Vec<_> = jq_mutex
        .iter()
        .filter(|(_, _, config)| match config {
            ClientMessage::JobConfig { cluster_size, .. } => {
                (*cluster_size as usize) < nodepool.active.load(Ordering::SeqCst)
            }
            _ => false,
        })
        .enumerate().map(|(idx, _)| idx).collect();
    
    return jq_filter; 
}

/// Using an index, this function will remove the required job from the job queue. This is so that 
/// it gives an ownership of the data to the caller of the function.
pub fn get_job(job_queue: &JobQueue, index: usize) -> Option<(ObjectId, DatasetPair, ClientMessage)> {
    let mut jq_mutex = job_queue
    .lock()
    .unwrap();

    jq_mutex.remove(index)
} 

/// Puts a job back in the job queue if it is not being executed. This will place it in a location 
/// specified by the index parameter. This will be the place in the job queue that it 
/// previously was.
pub fn put_job(job_queue: &JobQueue, index: usize, job: (ObjectId, DatasetPair, ClientMessage)) {
    let mut jq_mutex = job_queue
    .lock()
    .unwrap();

    jq_mutex.insert(index, job);
}

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node. The node will then do computation
/// on that dataset and will read in information from comp node.
pub async fn run(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    job_queue: JobQueue,
) -> Result<()> {
    log::info!("Job End Running");

    loop {
        let jq_filter = filter_jobs(&job_queue, &nodepool);

        for index in jq_filter {
            let (project_id, msg, config) = match get_job(&job_queue, index) {
                Some(job) => job,
                None => break
            };

            let data = msg
                .train
                .trim()
                .split('\n')
                .filter(|_| thread_rng().gen::<f64>() < INCLUSION_PROBABILITY)
                .chain(msg.predict.trim().split('\n').skip(1))
                .collect::<Vec<_>>()
                .join("\n");

            let mut columns = infer_dataset_columns(&data).unwrap();

            let cluster = match nodepool.build_cluster(config.anonymise(&columns)).await {
                Some(c) => c,
                _ => {
                    put_job(&job_queue, index, (project_id, msg, config));
                    continue;
                },
            };

            let mut train = msg.train.trim().split('\n').collect::<Vec<_>>();
            let headers = train.remove(0);
            let mut validation = Vec::new();
            let test = msg.predict.trim().split('\n').skip(1).collect::<Vec<_>>();

            let (prediction_column, prediction_type) = match config {
                ClientMessage::JobConfig {
                    ref prediction_column,
                    prediction_type,
                    ..
                } => (prediction_column.to_string(), prediction_type),
                _ => (
                    headers.split(',').last().unwrap().to_string(),
                    PredictionType::Classification,
                ),
            };

            if prediction_type == PredictionType::Classification {
                columns.insert(
                    prediction_column.clone(),
                    Column::categorical(&prediction_column, &data),
                );
            }

            for _ in 0..VALIDATION_SIZE {
                validation.push(train.swap_remove(thread_rng().gen_range(0..train.len())));
            }
            log::info!("Built validation data");

            let (bags, validation_ans, prediction_rids) = prepare_cluster(
                &cluster,
                headers,
                train,
                test,
                validation,
                &columns,
                prediction_column,
            );
            let info = ClusterInfo {
                project_id: project_id.clone(),
                columns: columns.clone(),
                config: config.clone(),
                validation_ans: validation_ans.clone(),
                prediction_rids: prediction_rids.clone(),
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

/// Function will take all data for a job and will bag the data to prepare for it 
/// being distributed among the models. This will return the data being sent to each 
/// model, as well as the rids of each prediction example in the test data to enable 
/// validation of results, as well as the validation answers.
pub fn prepare_cluster(
    cluster: &HashMap<String, Arc<RwLock<TcpStream>>>,
    headers: &str,
    train: Vec<&str>,
    test: Vec<&str>,
    validation: Vec<&str>,
    columns: &Columns,
    prediction_column: String,
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
        let model_train: Vec<_> = train
            .choose_multiple(&mut thread_rng(), TRAINING_BAG_SIZE)
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

        let mut anon_valid_ans: Vec<_> = anon_valid_ans.trim().split("\n").collect();
        let mut anon_valid: Vec<String> = Vec::new();

        anon_valid_ans.remove(0);
        let headers = format!("record_id,{}", headers);
        // Remove validation answers and record them for evaluation
        for (record, id) in anon_valid_ans.iter().zip(valid_rids.iter()) {
            let values: Vec<_> = record.split(',').zip(headers.split(',')).collect();
            let anon_ans = values
                .iter()
                .filter_map(|(v, h)| (*h == prediction_column).then(|| *v))
                .next()
                .unwrap()
                .to_string();
            let ans = columns
                .get(&prediction_column)
                .unwrap()
                .deanonymise(anon_ans)
                .unwrap();

            validation_ans.insert((key.clone(), id.to_owned()), ans.to_owned());
            anon_valid.push(
                values
                    .iter()
                    .map(|(v, h)| if *h != prediction_column { *v } else { "" })
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        let mut anon_valid: Vec<&str> = anon_valid.iter().map(|s| s.as_ref()).collect();
        let mut anon_test = anon_test.trim().split("\n").collect::<Vec<_>>();

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
            dcl_protcol(
                np_clone,
                database_clone,
                model_id,
                dcn_stream,
                info_clone,
                cc_clone,
                train_predict,
                wbm_clone,
            )
            .await
            .unwrap();
        });
    }

    let project_id = info.project_id.clone();

    cc.notify.notified().await;
    log::info!("All Jobs Complete!");

    let (weights, predictions) =
        ml::weight_predictions(&wbm.get_predictions(), &wbm.get_errors(), &info);

    // TODO: reimburse clients based on weights
    log::info!("Model weights: {:?}", weights);

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

    let csv: String = predictions.join("\n");

    write_predictions(database.clone(), info.project_id, csv.as_bytes())
        .await
        .unwrap_or_else(|error| log::error!("Error writing predictions to DB: {}", error));

    change_status(database, project_id, Status::Complete).await?;
    Ok(())
}

/// Function to execute DCL protocol
pub async fn dcl_protcol(
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

    let predictions = deanonymise_dataset(&anonymised_predictions, &info.columns).unwrap();

    if let Some((model_predictions, model_error)) =
        ml::evaluate_model(&model_id, &predictions, &info)
    {
        write_back.write_error(model_id.clone(), Some(model_error));
        write_back.write_predictions(model_id.clone(), model_predictions);
        log::info!("(Node: {}) Computed Data: {}", &model_id, predictions);
    } else {
        log::info!("(Node: {}) Failed to respond to all examples", &model_id);
        write_back.write_error(model_id.clone(), None);
    }

    increment_run_count(database, &model_id).await?;

    nodepool.end(&model_id).await?;

    cluster_control.decrement().await;

    Ok(())
}

/// Writes predictions back to the Mongo database for long term storage.
pub async fn write_predictions(
    database: Arc<Database>,
    id: ObjectId,
    dataset: &[u8],
) -> Result<()> {
    let predictions = database.collection("predictions");

    // Compress the data and make a new struct instance
    let compressed = compress_bytes(dataset)?;
    let prediction = Prediction::new(id, compressed);

    // Convert to a document and insert it
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
