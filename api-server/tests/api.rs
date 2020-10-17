use std::str::FromStr;

use bson::document::Document;
use bson::oid::ObjectId;
use serde::Deserialize;
use tide::http::Response;

use dodona::config::Environment;

#[derive(Deserialize)]
struct AuthResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

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
fn initialise() {
    INIT.call_once(|| {
        async_std::task::block_on(async {
            // Setup the environment variables
            let config = dodona::config::ConfigFile::from_file("config.toml");
            let resolved = config.resolve(Environment::Testing);
            resolved.populate_environment();

            // Connect to the database
            let conn_str = std::env::var("CONN_STR").expect("CONN_STR must be set");
            let client = mongodb::Client::with_uri_str(&conn_str).await.unwrap();
            let database = client.database("sybl");
            let collection_names = database.list_collection_names(None).await.unwrap();

            // Delete all records currently in the database
            for name in collection_names {
                let collection = database.collection(&name);
                collection.delete_many(Document::new(), None).await.unwrap();
            }

            // Insert some test users
            let peppered = format!("password{}", std::env::var("PEPPER").unwrap());
            let pbkdf2_iterations =
                u32::from_str(&std::env::var("PBKDF2_ITERATIONS").unwrap()).unwrap();
            let hash = pbkdf2::pbkdf2_simple(&peppered, pbkdf2_iterations).unwrap();

            let matthew = bson::doc! {
                "email": "matthewsmith@email.com",
                "password": hash,
                "first_name": "Matthew",
                "last_name": "Smith",
            };
            let delete = bson::doc! {
                "email": "delete@me.com",
                "password": "password",
                "first_name": "Delete",
                "last_name": "Me",
            };

            let users = database.collection("users");
            users.insert_one(matthew, None).await.unwrap();
            users.insert_one(delete, None).await.unwrap();
        });
    });
}

fn build_json_request(url: &str, body: &str) -> tide::http::Request {
    let url = tide::http::Url::parse(url).unwrap();
    let mut req = tide::http::Request::new(tide::http::Method::Post, url);

    req.set_body(body);
    req.set_content_type(tide::http::mime::JSON);

    req
}

#[async_std::test]
async fn users_can_register() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"
    {
        "email": "johnsmith@email.com",
        "password": "password",
        "firstName": "John",
        "lastName": "Smith"
    }"#;
    let req = build_json_request("localhost:/api/users/new", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: Result<AuthResponse, _> = res.body_json().await;
    assert!(body.is_ok());

    Ok(())
}

#[async_std::test]
async fn users_cannot_register_twice() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"
    {
        "email": "matthewsmith@email.com",
        "password": "password",
        "firstName": "Matthew",
        "lastName": "Smith"
    }"#;
    let req = build_json_request("localhost:/api/users/new", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: AuthResponse = res.body_json().await?;
    assert_eq!(body.token, "null");

    Ok(())
}

#[async_std::test]
async fn users_can_login() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"{
        "email": "matthewsmith@email.com",
        "password": "password"
    }"#;
    let req = build_json_request("localhost:/api/users/login", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: Result<AuthResponse, _> = res.body_json().await;
    assert!(body.is_ok());

    Ok(())
}

#[async_std::test]
async fn users_cannot_login_without_correct_password() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"{
        "email": "matthewsmith@email.com",
        "password": "incorrect"
    }"#;
    let req = build_json_request("localhost:/api/users/login", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: AuthResponse = res.body_json().await?;

    assert_eq!(body.token, "null");

    Ok(())
}

#[async_std::test]
async fn users_cannot_login_without_correct_email() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"{
        "email": "mattsmith@email.com",
        "password": "password"
    }"#;
    let req = build_json_request("localhost:/api/users/login", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: AuthResponse = res.body_json().await?;

    assert_eq!(body.token, "null");

    Ok(())
}

#[async_std::test]
async fn filter_finds_given_user_and_no_others() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"{"email": "matthewsmith@email.com"}"#;
    let req = build_json_request("localhost:/api/users/filter", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let users: Vec<User> = res.body_json().await?;

    assert_eq!(users.len(), 1);

    let found = &users[0];

    assert_eq!("Matthew", found.first_name);
    assert_eq!("Smith", found.last_name);

    Ok(())
}

#[async_std::test]
async fn non_existent_users_are_not_found() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    let body = r#"{"email": "nonexistent@email.com"}"#;
    let req = build_json_request("localhost:/api/users/filter", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let users: Vec<User> = res.body_json().await?;
    assert!(users.is_empty());

    Ok(())
}

#[async_std::test]
async fn users_can_be_deleted() -> tide::Result<()> {
    initialise();
    let app = dodona::build_server().await;

    // Find the user
    let body = r#"{"email": "delete@me.com"}"#;
    let req = build_json_request("localhost:/api/users/filter", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let users: Vec<User> = res.body_json().await?;
    let user = &users[0];

    // Delete the user
    let body = format!(r#"{{"id": "{}"}}"#, user.id.as_ref().unwrap().to_string());
    let req = build_json_request("localhost:/api/users/delete", &body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body = res.body_string().await?;
    let expected = r#"{"status":"deleted"}"#;

    assert_eq!(body, expected);

    Ok(())
}
