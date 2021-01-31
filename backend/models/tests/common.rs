use std::str::FromStr;

use mongodb::bson::{self, document::Document, oid::ObjectId};

use config::Environment;
use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::models::ClientModel;
use models::predictions::Prediction;
use models::projects::Project;
use models::users::{Client, User};

use utils::Columns;

// Hardcoded random identifiers for various tests
pub static USER_ID: &str = "60129e499dc3cc785ea1078b";
pub static CLIENT_USER_ID: &str = "60129e584e8a7634fd3f897f";

pub static PROJECT_ID: &str = "60129e654e8a7634fd3f8981";
pub static PROJECT_ID_2: &str = "60129e6c4e8a7634fd3f8982";

pub static DATASET_ID: &str = "60129e794e8a7634fd3f8983";
pub static DATASET_DETAILS_ID: &str = "60129e7f4e8a7634fd3f8984";

pub static DATASET_ID_2: &str = "60129e844e8a7634fd3f8985";
pub static DATASET_DETAILS_ID_2: &str = "60129e8b4e8a7634fd3f8986";

pub static PREDICTION_ID: &str = "60129e904e8a7634fd3f8987";

pub static MODEL_ID: &str = "60129e954e8a7634fd3f8988";

/// Allows for the setup of the database prior to testing.
static MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Defines the initialisation function for the tests.
///
/// This will clean the database and insert some basic data for testing purposes. It should be
/// called at the beginning of every test function, and the `std::once::Once` will ensure that it
/// is only ever called once.
///
/// As the database can be initialised before running, this allows tests to be run in any order
/// provided they don't require the result of a previous test.
pub async fn initialise() -> (mongodb::Database, tokio::sync::MutexGuard<'static, ()>) {
    // Acquire the mutex
    let lock = MUTEX.lock().await;

    // Check whether this is the first time being run
    let config = config::ConfigFile::from_filesystem();
    let resolved = config.resolve(Environment::Testing);
    resolved.populate_environment();

    // Connect to the database
    let conn_str = std::env::var("CONN_STR").expect("CONN_STR must be set");

    // Ensure that we aren't using the Atlas instance
    assert!(
        !conn_str.starts_with("mongodb+srv"),
        "Please setup a local MongoDB instance for running the tests"
    );
    let client = mongodb::Client::with_uri_str(&conn_str).await.unwrap();
    let database = client.database("sybl");
    let collection_names = database.list_collection_names(None).await.unwrap();

    // Delete all records currently in the database
    for name in collection_names {
        let collection = database.collection(&name);
        collection.delete_many(Document::new(), None).await.unwrap();
    }

    // Insert some test data
    insert_test_users(&database).await;
    insert_test_projects(&database).await;
    insert_test_data(&database).await;
    insert_test_predictions(&database).await;
    insert_test_models(&database).await;

    (database, lock)
}

fn create_user_with_id(
    id: &str,
    email: &str,
    hash: &str,
    first_name: &str,
    last_name: &str,
) -> bson::Document {
    let mut user = User::new(email, hash, first_name, last_name);

    user.id = ObjectId::with_string(id).unwrap();

    bson::ser::to_document(&user).unwrap()
}

fn create_project_with_id(id: &str, name: &str, desc: &str, uid: &str) -> bson::Document {
    let mut project = Project::new(name, desc, ObjectId::with_string(uid).unwrap());

    project.id = ObjectId::with_string(id).unwrap();

    bson::ser::to_document(&project).unwrap()
}

fn create_data_with_id(id: &str, det_id: &str, pid: &str) -> (bson::Document, bson::Document) {
    let data = "data".as_bytes();
    let pred = "pred".as_bytes();

    let mut details = DatasetDetails::new(
        "name".to_string(),
        ObjectId::with_string(pid).unwrap(),
        "data_head".to_string(),
        Columns::default(),
    );

    details.id = ObjectId::with_string(det_id).unwrap();

    let mut dataset = Dataset::new(
        ObjectId::with_string(pid).unwrap(),
        data.to_vec(),
        pred.to_vec(),
    );

    dataset.id = ObjectId::with_string(id).unwrap();

    (
        bson::ser::to_document(&dataset).unwrap(),
        bson::ser::to_document(&details).unwrap(),
    )
}

fn create_model_with_id(id: &str, name: &str, cid: &str) -> bson::Document {
    let challenge = "challenge".as_bytes().to_vec();
    let mut model = ClientModel::new(
        ObjectId::with_string(cid).unwrap(),
        name.to_string(),
        challenge,
    );

    model.id = ObjectId::with_string(id).unwrap();

    bson::ser::to_document(&model).unwrap()
}

fn create_pred_with_id(id: &str, pid: &str) -> bson::Document {
    let pid = ObjectId::with_string(pid).unwrap();
    let id = ObjectId::with_string(id).unwrap();

    let mut prediction = Prediction::new(pid, "pred".as_bytes().to_vec());

    prediction.id = id;

    bson::ser::to_document(&prediction).unwrap()
}

async fn insert_test_users(database: &mongodb::Database) {
    let matthew = create_user_with_id(
        USER_ID,
        "matthewsmith@email.com",
        "password",
        "Matthew",
        "Smith",
    );
    let client = create_user_with_id(
        CLIENT_USER_ID,
        "client@model.com",
        "password",
        "Create",
        "Project",
    );
    let users = database.collection("users");
    users.insert_one(matthew, None).await.unwrap();
    users.insert_one(client, None).await.unwrap();
}

async fn insert_test_projects(database: &mongodb::Database) {
    let project = create_project_with_id(PROJECT_ID, "Test Project", "Test Description", USER_ID);
    let project_2 =
        create_project_with_id(PROJECT_ID_2, "Test Project", "Project the second", USER_ID);

    let projects = database.collection("projects");
    projects.insert_one(project, None).await.unwrap();
    projects.insert_one(project_2, None).await.unwrap();
}

async fn insert_test_data(database: &mongodb::Database) {
    let (dataset, details) = create_data_with_id(DATASET_ID, DATASET_DETAILS_ID, PROJECT_ID);
    let (dataset_2, details_2) =
        create_data_with_id(DATASET_ID_2, DATASET_DETAILS_ID_2, PROJECT_ID_2);

    let datasets = database.collection("datasets");
    datasets.insert_one(dataset, None).await.unwrap();
    datasets.insert_one(dataset_2, None).await.unwrap();

    let details_col = database.collection("dataset_details");
    details_col.insert_one(details, None).await.unwrap();
    details_col.insert_one(details_2, None).await.unwrap();
}

async fn insert_test_predictions(database: &mongodb::Database) {
    let prediction = create_pred_with_id(PREDICTION_ID, PROJECT_ID);

    let predictions = database.collection("predictions");
    predictions.insert_one(prediction, None).await.unwrap();
}

async fn insert_test_models(database: &mongodb::Database) {
    let model = create_model_with_id(MODEL_ID, "model_name", CLIENT_USER_ID);

    let models = database.collection("models");
    models.insert_one(model, None).await.unwrap();
}
