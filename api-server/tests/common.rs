use std::str::FromStr;

use chrono::TimeZone;
use mongodb::bson::{self, document::Document, oid::ObjectId};

use dodona::config::Environment;

// Hardcoded random identifiers for various tests
pub static MAIN_USER_ID: &str = "5f8ca1a80065f27b0089e8b5";
pub static CREATES_PROJECT_UID: &str = "5f8d7b4f0017036400d60cab";
pub static NON_EXISTENT_USER_ID: &str = "5f8de85300eb281e00306b0b";

pub static MAIN_PROJECT_ID: &str = "5f8ca1a80065f27c0089e8b5";
pub static USERLESS_PROJECT_ID: &str = "5f8ca1a80065f27b0089e8b6";
pub static NON_EXISTENT_PROJECT_ID: &str = "5f8ca1a80065f27b0089e8a5";

/// Allows for the setup of the database prior to testing.
static INIT: std::sync::Once = std::sync::Once::new();

/// Defines the initialisation function for the tests.
///
/// This will clean the database and insert some basic data for testing purposes. It should be
/// called at the beginning of every test function, and the `std::once::Once` will ensure that it
/// is only ever called once.
///
/// As the database can be initialised before running, this allows tests to be run in any order
/// provided they don't require the result of a previous test.
pub fn initialise() {
    INIT.call_once(|| {
        async_std::task::block_on(async {
            // Setup the environment variables
            let config = dodona::config::ConfigFile::from_file("config.toml");
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
        });
    });
}

async fn insert_test_users(database: &mongodb::Database) {
    let peppered = format!("password{}", std::env::var("PEPPER").unwrap());
    let pbkdf2_iterations = u32::from_str(&std::env::var("PBKDF2_ITERATIONS").unwrap()).unwrap();
    let hash = pbkdf2::pbkdf2_simple(&peppered, pbkdf2_iterations).unwrap();

    let matthew = bson::doc! {
        "_id": ObjectId::with_string(MAIN_USER_ID).unwrap(),
        "email": "matthewsmith@email.com",
        "password": hash,
        "first_name": "Matthew",
        "last_name": "Smith",
        "api_key": "",
    };
    let delete = bson::doc! {
        "email": "delete@me.com",
        "password": "password",
        "first_name": "Delete",
        "last_name": "Me",
        "api_key": "",
    };
    let creates_project = bson::doc! {
        "_id": ObjectId::with_string(CREATES_PROJECT_UID).unwrap(),
        "email": "creates@projects.com",
        "password": "password",
        "first_name": "Create",
        "last_name": "Project",
        "api_key": "",
    };

    let users = database.collection("users");
    users.insert_one(matthew, None).await.unwrap();
    users.insert_one(delete, None).await.unwrap();
    users.insert_one(creates_project, None).await.unwrap();
}

async fn insert_test_projects(database: &mongodb::Database) {
    let project = bson::doc! {
        "_id": ObjectId::with_string(MAIN_PROJECT_ID).unwrap(),
        "name": "Test Project",
        "description": "Test Description",
        "date_created": bson::Bson::DateTime(chrono::Utc.timestamp_millis(0)),
        "user_id": ObjectId::with_string(MAIN_USER_ID).unwrap(),
        "status": "Ready"
    };
    let userless = bson::doc! {
        "_id": ObjectId::with_string(USERLESS_PROJECT_ID).unwrap(),
        "name": "Test Project",
        "description": "Test Description",
        "date_created": bson::Bson::DateTime(chrono::Utc.timestamp_millis(0)),
        "user_id": ObjectId::with_string(NON_EXISTENT_USER_ID).unwrap(),
        "status": "Ready"
    };

    let projects = database.collection("projects");
    projects.insert_one(project, None).await.unwrap();
    projects.insert_one(userless, None).await.unwrap();
}

pub fn build_json_request(url: &str, body: &str) -> tide::http::Request {
    let full_url = format!("localhost:{}", url);
    let url = tide::http::Url::parse(&full_url).unwrap();
    let mut req = tide::http::Request::new(tide::http::Method::Post, url);

    req.set_body(body);
    req.set_content_type(tide::http::mime::JSON);

    req
}

pub fn build_json_put_request(url: &str, body: &str) -> tide::http::Request {
    let full_url = format!("localhost:{}", url);
    let url = tide::http::Url::parse(&full_url).unwrap();
    let mut req = tide::http::Request::new(tide::http::Method::Put, url);

    req.set_body(body);
    req.set_content_type(tide::http::mime::JSON);

    req
}
