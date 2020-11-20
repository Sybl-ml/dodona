use serde::Deserialize;
use tide::http::Response;

use models::users::User;

mod common;

#[derive(Deserialize)]
struct AuthResponse {
    pub token: String,
}

#[async_std::test]
async fn users_can_register() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"
    {
        "email": "johnsmith@email.com",
        "password": "password",
        "firstName": "John",
        "lastName": "Smith"
    }"#;
    let req = common::build_json_request("/api/users/new", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: Result<AuthResponse, _> = res.body_json().await;
    assert!(body.is_ok());

    Ok(())
}

#[async_std::test]
async fn users_cannot_register_twice() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"
    {
        "email": "matthewsmith@email.com",
        "password": "password",
        "firstName": "Matthew",
        "lastName": "Smith"
    }"#;
    let req = common::build_json_request("/api/users/new", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: AuthResponse = res.body_json().await?;
    assert_eq!(body.token, "null");

    Ok(())
}

#[async_std::test]
async fn users_can_login() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{
        "email": "matthewsmith@email.com",
        "password": "password"
    }"#;
    let req = common::build_json_request("/api/users/login", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: Result<AuthResponse, _> = res.body_json().await;
    assert!(body.is_ok());

    Ok(())
}

#[async_std::test]
async fn users_cannot_login_without_correct_password() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{
        "email": "matthewsmith@email.com",
        "password": "incorrect"
    }"#;
    let req = common::build_json_request("/api/users/login", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: AuthResponse = res.body_json().await?;

    assert_eq!(body.token, "null");

    Ok(())
}

#[async_std::test]
async fn users_cannot_login_without_correct_email() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{
        "email": "mattsmith@email.com",
        "password": "password"
    }"#;
    let req = common::build_json_request("/api/users/login", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body: AuthResponse = res.body_json().await?;

    assert_eq!(body.token, "null");

    Ok(())
}

#[async_std::test]
async fn filter_finds_given_user_and_no_others() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"email": "matthewsmith@email.com"}"#;
    let req = common::build_json_request("/api/users/filter", body);

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
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"email": "nonexistent@email.com"}"#;
    let req = common::build_json_request("/api/users/filter", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let users: Vec<User> = res.body_json().await?;
    assert!(users.is_empty());

    Ok(())
}

#[async_std::test]
async fn users_can_be_deleted() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    // Find the user
    let body = r#"{"email": "delete@me.com"}"#;
    let req = common::build_json_request("/api/users/filter", body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let users: Vec<User> = res.body_json().await?;
    let user = &users[0];

    // Delete the user
    let body = format!(r#"{{"id": "{}"}}"#, user.id.as_ref().unwrap().to_string());
    let req = common::build_json_request("/api/users/delete", &body);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let body = res.body_string().await?;
    let expected = r#"{"status":"deleted"}"#;

    assert_eq!(body, expected);

    Ok(())
}
