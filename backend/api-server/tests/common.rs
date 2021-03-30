use std::env;
use std::str::FromStr;
use std::sync::Arc;

use mongodb::bson::{self, document::Document, oid::ObjectId, ser::to_document};

use api_server::{auth, State};
use config::Environment;
use models::job_performance::JobPerformance;
use models::projects::Project;
use models::users::{Client, User};

#[allow(unused_macros)]
macro_rules! api_with {
    ($method:path: $route:literal => $handler:path) => {
        test::init_service(
            App::new()
                .wrap(middleware::Logger::default())
                .data(common::initialise().await)
                .route($route, $method().to($handler)),
        )
        .await;
    };
    ($($method:path: $route:literal => $handler:path,)*) => {
        test::init_service(
            App::new()
                .wrap(middleware::Logger::default())
                .data(common::initialise().await)
                $(.route($route, $method().to($handler)))*,
        )
        .await;
    };
}

// Hardcoded random identifiers for various tests
pub static MAIN_USER_ID: &str = "5f8ca1a80065f27b0089e8b5";
pub static DELETE_UID: &str = "5fbe3239ea6cfda08a459622";
pub static CREATES_PROJECT_UID: &str = "5f8d7b4f0017036400d60cab";
pub static NON_EXISTENT_USER_ID: &str = "5f8de85300eb281e00306b0b";
pub static DELETES_PROJECT_UID: &str = "5fb2b3fa9d524e99ac7f1c40";

pub static MAIN_CLIENT_ID: &str = "602bfa774f986d0e58618187";

pub static MAIN_PROJECT_ID: &str = "5f8ca1a80065f27c0089e8b5";
pub static USERLESS_PROJECT_ID: &str = "5f8ca1a80065f27b0089e8b6";
#[allow(dead_code)]
pub static NON_EXISTENT_PROJECT_ID: &str = "5f8ca1a80065f27b0089e8a5";
pub static OVERWRITTEN_DATA_PROJECT_ID: &str = "5fb784e4ead1758e1ce67bcd";
pub static DELETABLE_PROJECT_ID: &str = "5fb2b4049d524e99ac7f1c41";
pub static EDITABLE_PROJECT_ID: &str = "5fb2c4e4b4b7becc1e81e278";
pub static MODEL_ID: &str = "5fb2c4e4b4b7becc1e81e279";

/// Allows for the setup of the database prior to testing.
static MUTEX: tokio::sync::Mutex<bool> = tokio::sync::Mutex::const_new(true);

/// Defines the initialisation function for the tests.
///
/// This will clean the database and insert some basic data for testing purposes. It should be
/// called at the beginning of every test function, and the `std::once::Once` will ensure that it
/// is only ever called once.
///
/// As the database can be initialised before running, this allows tests to be run in any order
/// provided they don't require the result of a previous test.
pub async fn initialise() -> State {
    // Acquire the mutex
    let mut lock = MUTEX.lock().await;

    // Check whether this is the first time being run
    if *lock {
        let config = config::ConfigFile::from_filesystem();
        let resolved = config.resolve(Environment::Testing);
        resolved.populate_environment();

        // Connect to the database
        let conn_str = std::env::var("CONN_STR").expect("CONN_STR must be set");
        let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("sybl"));

        // Ensure that we aren't using the Atlas instance
        assert!(
            !conn_str.starts_with("mongodb+srv"),
            "Please setup a local MongoDB instance for running the tests"
        );
        let client = mongodb::Client::with_uri_str(&conn_str).await.unwrap();
        let database = client.database(&database_name);
        let collection_names = database.list_collection_names(None).await.unwrap();

        // Delete all records currently in the database
        for name in collection_names {
            let collection = database.collection(&name);
            collection.delete_many(Document::new(), None).await.unwrap();
        }

        // Insert some test data
        insert_test_users(&database).await;
        insert_test_clients(&database).await;
        insert_test_projects(&database).await;
        insert_test_job_performances(&database).await;

        // Update the lock
        *lock = false;
    }

    build_app_state().await
}

pub async fn build_app_state() -> State {
    let conn_str = std::env::var("CONN_STR").expect("CONN_STR must be set");
    let pepper = std::env::var("PEPPER").expect("PEPPER must be set");
    let pbkdf2_iterations =
        std::env::var("PBKDF2_ITERATIONS").expect("PBKDF2_ITERATIONS must be set");

    let client = mongodb::Client::with_uri_str(&conn_str).await.unwrap();
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("sybl"));

    let database = Arc::new(client.database(&database_name));

    State {
        database: Arc::clone(&database),
        pepper: Arc::new(pepper.clone()),
        pbkdf2_iterations: u32::from_str(&pbkdf2_iterations)
            .expect("PBKDF2_ITERATIONS must be parseable as an integer"),
    }
}

pub fn get_bearer_token(identifier: &str) -> String {
    // Create a user for authentication
    let encoded = auth::Claims::create_token(ObjectId::with_string(identifier).unwrap()).unwrap();

    format!("Bearer {}", encoded)
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

    to_document(&user).unwrap()
}

fn create_client_with_id(
    id: &str,
    email: &str,
    hash: &str,
    first_name: &str,
    last_name: &str,
) -> (bson::Document, bson::Document) {
    let mut user = User::new(email, hash, first_name, last_name);

    user.id = ObjectId::with_string(id).unwrap();

    // Upgrade them to a client
    let client = Client::new(user.id.clone(), crypto::encoded_key_pair().1);
    user.client = true;

    let user_document = to_document(&user).unwrap();
    let client_document = to_document(&client).unwrap();

    (user_document, client_document)
}

fn create_project_with_id(id: &str, name: &str, desc: &str, uid: &str) -> bson::Document {
    let mut project = Project::new(
        name,
        desc,
        vec![bson::bson!("test"), bson::bson!("tag")],
        ObjectId::with_string(uid).unwrap(),
    );

    project.id = ObjectId::with_string(id).unwrap();

    to_document(&project).unwrap()
}

async fn insert_test_users(database: &mongodb::Database) {
    let peppered = format!("password{}", std::env::var("PEPPER").unwrap());
    let pbkdf2_iterations = u32::from_str(&std::env::var("PBKDF2_ITERATIONS").unwrap()).unwrap();
    let hash = crypto::hash_password(&peppered, pbkdf2_iterations).unwrap();

    let matthew = create_user_with_id(
        MAIN_USER_ID,
        "matthewsmith@email.com",
        &hash,
        "Matthew",
        "Smith",
    );
    let delete = create_user_with_id(DELETE_UID, "delete@me.com", "password", "Delete", "Me");
    let creates_project = create_user_with_id(
        CREATES_PROJECT_UID,
        "creates@projects.com",
        "password",
        "Create",
        "Project",
    );
    let deletes_project = create_user_with_id(
        DELETES_PROJECT_UID,
        "deletes@projects.com",
        "password",
        "Delete",
        "Project",
    );

    let users = database.collection("users");
    users.insert_one(matthew, None).await.unwrap();
    users.insert_one(delete, None).await.unwrap();
    users.insert_one(creates_project, None).await.unwrap();
    users.insert_one(deletes_project, None).await.unwrap();
}

async fn insert_test_clients(database: &mongodb::Database) {
    let peppered = format!("password{}", std::env::var("PEPPER").unwrap());
    let pbkdf2_iterations = u32::from_str(&std::env::var("PBKDF2_ITERATIONS").unwrap()).unwrap();
    let hash = crypto::hash_password(&peppered, pbkdf2_iterations).unwrap();

    let (user, client) =
        create_client_with_id(MAIN_CLIENT_ID, "client@sybl.com", &hash, "client", "user");

    let users = database.collection("users");
    users.insert_one(user, None).await.unwrap();

    let clients = database.collection("clients");
    clients.insert_one(client, None).await.unwrap();
}

async fn insert_test_projects(database: &mongodb::Database) {
    let project = create_project_with_id(
        MAIN_PROJECT_ID,
        "Test Project",
        "Test Description",
        MAIN_USER_ID,
    );
    let userless = create_project_with_id(
        USERLESS_PROJECT_ID,
        "Test Project",
        "userless",
        NON_EXISTENT_USER_ID,
    );
    let overwritten_data = create_project_with_id(
        OVERWRITTEN_DATA_PROJECT_ID,
        "Test Project",
        "Test Description",
        NON_EXISTENT_USER_ID,
    );
    let deletable = create_project_with_id(
        DELETABLE_PROJECT_ID,
        "Delete Project",
        "Test Description",
        DELETES_PROJECT_UID,
    );
    let editable =
        create_project_with_id(EDITABLE_PROJECT_ID, "Edit Project", "edit me", MAIN_USER_ID);

    let projects = database.collection("projects");
    projects.insert_one(project, None).await.unwrap();
    projects.insert_one(userless, None).await.unwrap();
    projects.insert_one(overwritten_data, None).await.unwrap();
    projects.insert_one(deletable, None).await.unwrap();
    projects.insert_one(editable, None).await.unwrap();
}

async fn insert_test_job_performances(database: &mongodb::Database) {
    let job_performances = database.collection("job_performances");
    let results: Vec<f64> = vec![0.6, 0.5, 0.4];
    let time = chrono::Utc::now();

    let project_id = ObjectId::with_string(MAIN_PROJECT_ID).unwrap();
    let model_id = ObjectId::with_string(MODEL_ID).unwrap();

    let performances = results.iter().enumerate().map(|(i, r)| {
        let mut job_performance = JobPerformance::new(project_id.clone(), model_id.clone(), *r);

        let offset = chrono::Duration::milliseconds(i as i64);
        job_performance.date_created = bson::DateTime(time + offset);

        job_performance
    });

    for instance in performances {
        let serialized = to_document(&instance).unwrap();
        job_performances.insert_one(serialized, None).await.unwrap();
    }
}
