//! Part of DCL that takes a DCN and a dataset and comunicates with node

use std::cmp::max;
use std::collections::HashMap;
use std::convert::From;
use std::env;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use mongodb::{
    bson::{doc, from_document, oid::ObjectId},
    Database,
};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{Notify, RwLock};
use tokio::time::{timeout, Instant};

use crate::node_end::NodePool;
use crate::JobControl;
use messages::{
    kafka_message, ClientCompleteMessage, ClientMessage, ReadLengthPrefix, WriteLengthPrefix,
};
use models::gridfs;
use models::jobs::PredictionType;
use models::jobs::{Job, JobStatistics};
use models::predictions::Prediction;
use models::projects::{Project, Status};
use models::users::User;

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
    /// The job that is currently running
    pub job: Job,
    /// Validation results
    pub validation_ans: HashMap<(ModelID, String), String>,
    /// Test record IDs
    pub prediction_rids: HashMap<(ModelID, String), usize>,
    /// The amount of time each node is allowed to compute for
    pub node_computation_time: Duration,
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
    /// Vector of computation times for each model
    pub computation_time: Arc<Mutex<Vec<i64>>>,
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

    /// Function to write back computation time (secs)
    pub fn write_time(&self, time: i64) {
        let mut computation_time = self.computation_time.lock().unwrap();
        computation_time.push(time);
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

    /// Gets cloned version of errors
    pub fn get_average_job_time(&self) -> i64 {
        let computation_time = self.computation_time.lock().unwrap();
        if computation_time.len() > 0 {
            computation_time.iter().sum::<i64>() as i64 / computation_time.len() as i64
        } else {
            0
        }
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
    pub async fn decrement(&self) -> usize {
        let mut write_cc = self.counter.write().await;
        *write_cc -= 1;
        if *write_cc == 0 {
            self.notify.notify_one();
        }
        *write_cc
    }
}

/// The configuration for sending emails.
#[derive(Debug)]
pub struct Config {
    /// The address to send from.
    pub from_address: String,
    /// The name to send from.
    pub from_name: String,
    /// The application specific password for Gmail.
    pub app_password: String,
}

impl Config {
    /// Builds a configuration from the environment variables.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            from_address: env::var("FROM_ADDRESS")?,
            from_name: env::var("FROM_NAME")?,
            app_password: env::var("APP_PASSWORD")?,
        })
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
    loop {
        let jq_filter = job_control.job_queue.filter(&nodepool.active);

        if jq_filter.is_empty() {
            job_control.notify.notified().await;
            continue;
        }

        for index in jq_filter {
            let (project_id, msg, job) = job_control.job_queue.remove(index);
            let config = &job.config;

            let data = msg
                .train
                .trim()
                .split('\n')
                .chain(msg.predict.trim().split('\n').skip(1))
                .collect::<Vec<_>>()
                .join("\n");

            let mut columns = infer_dataset_columns(&data).unwrap();

            if config.prediction_type == PredictionType::Classification {
                columns.insert(
                    config.prediction_column.clone(),
                    Column::categorical(&config.prediction_column, &data),
                );
            }

            // Anonymise the prediction column for the job
            let anonymised_config = config.anonymise(&columns);

            let cluster = match nodepool.build_cluster(anonymised_config).await {
                Some(c) => c,
                _ => {
                    log::warn!(
                        "Failed to build a cluster for project_id={}, requiring a configuration of: {:?}",
                        project_id,
                        config
                    );

                    job_control.job_queue.insert(index, (project_id, msg, job));

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

            for _ in 0..max(1, (train.len() as f64 * VALIDATION_SPLIT) as usize) {
                validation.push(train.swap_remove(thread_rng().gen_range(0..train.len())));
            }

            let (bags, validation_ans, prediction_rids) = prepare_cluster(
                &cluster,
                headers,
                &train,
                &test,
                &validation,
                &columns,
                &config.prediction_column,
            );

            let info = ClusterInfo {
                project_id: project_id.clone(),
                columns: columns.clone(),
                job: job.clone(),
                validation_ans: validation_ans.clone(),
                prediction_rids: prediction_rids.clone(),
                node_computation_time: Duration::from_secs(
                    (config.node_computation_time * 60) as u64,
                ),
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

        log::info!("No jobs could be completed at the moment, waiting for changes");
        job_control.notify.notified().await;
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
        log::info!("Bootstrapping dataset with key={}", key);

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

        log::trace!(
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

        log::trace!("IDs: {:?}\nAnonymised Test: {:?}", &test_rids, &anon_test);

        // Add record ids to validation
        let (anon_valid_ans, valid_rids) = generate_ids(&anon_valid);

        log::trace!(
            "IDs: {:?}\nAnonymised Valid: {:?}",
            valid_rids,
            anon_valid_ans
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

        log::trace!("Anonymised Test with Validation: {:?}", &final_anon_test);

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
            let wait = info_clone.node_computation_time;

            let future = dcl_protocol(
                np_clone,
                database_clone,
                &model_id,
                dcn_stream,
                info_clone,
                cc_clone,
                train_predict,
                wbm_clone,
            );

            if timeout(wait, future).await.is_err() {
                log::warn!("Model with id={} failed to respond in time", model_id);
            }
        });
    }

    let project_id = info.project_id.clone();

    cc.notify.notified().await;

    let (weights, predictions) =
        ml::weight_predictions(&wbm.get_predictions(), &wbm.get_errors(), &info);

    // TODO: reimburse clients based on weights
    log::info!("Model weights: {:?}", weights);

    for (model_id, weight) in &weights {
        let database_clone = Arc::clone(&database);
        reimburse(
            database_clone,
            &ObjectId::with_string(model_id).unwrap(),
            info.job.config.cost,
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
        .unwrap_or_else(|error| {
            log::error!("Failed to write predirections to the database: {}", error)
        });

    // Write job statistics to database
    let job_statistic = JobStatistics::new(info.job.id.clone(), wbm.get_average_job_time());
    let document = mongodb::bson::ser::to_document(&job_statistic)?;
    database
        .collection("job_statistics")
        .insert_one(document, None)
        .await?;

    change_status(&database, &project_id, Status::Complete).await?;

    // Mark the job as processed
    info.job.mark_as_processed(&database).await?;

    // Status has been updated to complete, so email the user
    if let Err(e) = email_user_on_project_finish(&database, &project_id).await {
        log::warn!(
            "Failed to email the user upon finishing processing of project_id={}: {}",
            project_id,
            e
        );
    }

    Ok(())
}

/// Decodes the incoming data and decompresses it.
///
/// Incoming data is expected to be compressed and then encoded with Base64, so this will simply
/// reverse the process.
fn decode_and_decompress(data: &str) -> Result<String> {
    let decoded = base64::decode(data)?;
    let decompressed = utils::compress::decompress_data(&decoded)?;
    Ok(String::from_utf8(decompressed)?)
}

/// Function to execute DCL protocol
pub async fn dcl_protocol(
    nodepool: Arc<NodePool>,
    database: Arc<Database>,
    model_id: &str,
    stream: Arc<RwLock<TcpStream>>,
    info: ClusterInfo,
    cluster_control: ClusterControl,
    (train, predict): (String, String),
    write_back: WriteBackMemory,
) -> Result<()> {
    log::debug!("Sending a job to node with id={}", model_id);

    let mut dcn_stream = stream.write().await;

    let mut buffer = [0_u8; 1024];

    // Compress the data beforehand
    let dataset_message = ClientMessage::from_train_and_predict(&train, &predict);

    // Start a timer to track execution time and send the data across
    let start = Instant::now();
    dcn_stream.write(&dataset_message.as_bytes()).await.unwrap();

    // TODO: Propagate this error forward to the frontend so that it can say a node has failed
    let prediction_message =
        match ClientMessage::from_stream(dcn_stream.deref_mut(), &mut buffer).await {
            Ok(pm) => pm,
            Err(error) => {
                nodepool.update_node_alive(&model_id, false).await;
                cluster_control.decrement().await;

                log::error!(
                    "Node with id={} failed to deal with predictions: {}",
                    model_id,
                    error
                );

                return Ok(());
            }
        };

    // Stop the timer and record how long was spent processing
    let processing_time_secs = (Instant::now() - start).as_secs();
    log::trace!(
        "model_id={} spent {:?} processing {} training bytes and {} predictions bytes",
        model_id,
        processing_time_secs,
        train.len(),
        predict.len()
    );

    // Ensure it is the right message and decode + decompress it
    let anonymised_predictions = match prediction_message {
        ClientMessage::Predictions(s) => decode_and_decompress(&s),
        _ => unreachable!(),
    };

    // Evaluate the model
    let mut model_success = true;
    let model_evaluation = anonymised_predictions.map(|preds| {
        deanonymise_dataset(preds.trim(), &info.columns)
            .and_then(|predictions| ml::evaluate_model(model_id, &predictions, &info))
    });

    // Check that we got valid predictions and they evaluated correctly
    if let Ok(Some((model_predictions, model_error))) = model_evaluation {
        log::info!(
            "Node with id={} produced {} rows of predictions",
            model_id,
            model_predictions.len()
        );

        write_back.write_error(model_id.to_owned(), Some(model_error));
        write_back.write_predictions(model_id.to_owned(), model_predictions);
        write_back.write_time(processing_time_secs as i64);
    } else {
        log::warn!("Node with id={} failed to respond correctly", model_id);
        model_success = false;
        write_back.write_error(model_id.to_owned(), None);
    }

    update_model_statistics(&database, &model_id, processing_time_secs).await?;
    nodepool.end(&model_id).await?;

    let remaining_nodes = cluster_control.decrement().await;
    let cluster_size = info.job.config.cluster_size as usize;
    // Produce message
    let message = ClientCompleteMessage {
        project_id: &info.project_id.to_string(),
        cluster_size,
        model_complete_count: cluster_size - remaining_nodes,
        success: model_success,
    };

    let message = serde_json::to_string(&message).unwrap();
    let projects = database.collection("projects");
    let filter = doc! {"_id": info.project_id};
    let update = if model_success {
        doc! {"$inc": {"status.Processing.model_success": 1}}
    } else {
        doc! {"$inc": {"status.Processing.model_err": 1}}
    };

    let doc = projects
        .find_one_and_update(filter, update, None)
        .await?
        .expect("Failed to find project in db");

    let project: Project = from_document(doc)?;
    let message_key = project.user_id.to_string();
    let topic = "project_updates";

    kafka_message::produce_message(&message, &message_key, &topic).await;

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

/// Updates the non-performance related statistics for the given model.
///
/// This increments the number of times it has been run and adds the amount of time it spent
/// processing the last job.
pub async fn update_model_statistics(
    database: &Arc<Database>,
    model_id: &str,
    processing_time_secs: u64,
) -> Result<()> {
    let models = database.collection("models");
    let object_id = ObjectId::with_string(model_id)?;

    log::debug!(
        "Incrementing run count for model with id={}, adding {} seconds of processing time",
        model_id,
        processing_time_secs
    );

    let query = doc! {"_id": &object_id};
    let update = doc! {
        "$inc": {
            "times_run": 1,
            "processing_time_secs": processing_time_secs as i64
        }
    };

    models.update_one(query, update, None).await?;

    Ok(())
}

/// Change the status of a project after it has been completed
pub async fn change_status(
    database: &Arc<Database>,
    project_id: &ObjectId,
    status: Status,
) -> Result<()> {
    let projects = database.collection("projects");

    log::info!("Setting status={:?} for project_id={}", status, project_id);

    projects
        .update_one(
            doc! { "_id": project_id},
            doc! {"$set": {"status": status}},
            None,
        )
        .await?;

    Ok(())
}

/// Emails the user to inform them the project has finished.
///
/// If any of the appropriate environment variables are not set (being the `FROM_ADDRESS`,
/// `FROM_NAME` and `APP_PASSWORD`), no emails will sent. Otherwise, this will retrieve the project
/// information from the database and the user who created it, before formatting an email and
/// sending it to them. This is currently hardcoded to use Google's mail servers and is largely
/// based on similar code that runs the email system for `blackboards.pl`.
pub async fn email_user_on_project_finish(
    database: &Arc<Database>,
    project_id: &ObjectId,
) -> Result<()> {
    let users = database.collection("users");
    let projects = database.collection("projects");

    // Find the project itself and convert it
    let document = projects
        .find_one(doc! { "_id": &project_id }, None)
        .await?
        .expect("Project did not exist in the database");

    let project = from_document::<Project>(document)?;

    // Find the user associated with the project and deserialize it
    let user_document = users
        .find_one(doc! { "_id": &project.user_id }, None)
        .await?
        .expect("Failed to find user associated with project");

    let user: User = from_document(user_document)?;

    // Get the configuration from the environment if it exists
    let email_config = Config::from_env()?;

    // Form the sender and receiver addresses, as well as the body
    let from = format!("{} <{}>", email_config.from_name, email_config.from_address);
    let to = format!("{} {} <{}>", user.first_name, user.last_name, user.email);
    let body = format!(
        r#"Hi {},

Your project '{}' has finished processing, and results are now available. You can view them at https://sybl.tech/dashboard/{}"#,
        user.first_name, project.name, project_id
    );

    // Build the message to send
    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject("Sybl Project Completion")
        .body(body)?;

    // Use the credentials from the configuration parsed earlier
    let creds = Credentials::new(email_config.from_address, email_config.app_password);

    // Open a remote connection to gmail
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    // Send the email itself
    mailer.send(email).await?;

    Ok(())
}
