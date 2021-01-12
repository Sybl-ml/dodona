use serde::Deserialize;

use models::users::User;

use actix_web::{middleware, test, web, App, Result};
use api_server::routes;
use mongodb::bson::{doc, document::Document};

mod common;

#[derive(Deserialize, Debug)]
struct AuthResponse {
    pub token: String,
}

#[actix_rt::test]
async fn users_can_register() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/new", web::post().to(routes::users::new)),
    )
    .await;

    let doc = doc! {
        "email": "johnsmith@email.com",
        "password": "password",
        "firstName": "John",
        "lastName": "Smith"
    };

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/new")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    let body: AuthResponse = test::read_body_json(res).await;
    assert!(body.token != "null");

    Ok(())
}

#[actix_rt::test]
async fn users_cannot_register_twice() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/new", web::post().to(routes::users::new)),
    )
    .await;

    let doc = doc! {
        "email": "matthewsmith@email.com",
        "password": "password",
        "firstName": "Matthew",
        "lastName": "Smith"
    };
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/new")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let body: AuthResponse = test::read_body_json(res).await;
    assert_eq!(body.token, "null");

    Ok(())
}

#[actix_rt::test]
async fn users_can_login() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/login", web::post().to(routes::users::login)),
    )
    .await;

    let doc = doc! {
        "email": "matthewsmith@email.com",
        "password": "password"
    };
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/login")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let body: AuthResponse = test::read_body_json(res).await;
    assert!(body.token != "null");

    Ok(())
}

#[actix_rt::test]
async fn users_cannot_login_without_correct_password() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/login", web::post().to(routes::users::login)),
    )
    .await;

    let doc = doc! {
        "email": "matthewsmith@email.com",
        "password": "incorrect"
    };
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/login")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::UNAUTHORIZED, res.status());

    Ok(())
}

#[actix_rt::test]
async fn users_cannot_login_without_correct_email() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/login", web::post().to(routes::users::login)),
    )
    .await;

    let doc = doc! {
        "email": "incorrect@email.com",
        "password": "passowrd"
    };
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/login")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::NOT_FOUND, res.status());

    Ok(())
}

#[actix_rt::test]
async fn filter_finds_given_user_and_no_others() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/filter", web::post().to(routes::users::filter)),
    )
    .await;

    let doc = doc! {"email": "matthewsmith@email.com"};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/filter")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let users: Vec<User> = test::read_body_json(res).await;

    assert_eq!(users.len(), 1);

    let found = &users[0];

    assert_eq!("Matthew", found.first_name);
    assert_eq!("Smith", found.last_name);

    Ok(())
}

#[actix_rt::test]
async fn non_existent_users_are_not_found() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/filter", web::post().to(routes::users::filter)),
    )
    .await;

    let doc = doc! {"email": "nonexistent@email.com"};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/filter")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let users: Vec<User> = test::read_body_json(res).await;
    assert!(users.is_empty());

    Ok(())
}

#[actix_rt::test]
async fn users_can_be_deleted() -> Result<()> {
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route("/api/users/filter", web::post().to(routes::users::filter))
            .route("/api/users/delete", web::post().to(routes::users::delete)),
    )
    .await;

    // Find the user
    let doc = doc! {"email": "delete@me.com"};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/filter")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let users: Vec<User> = test::read_body_json(res).await;
    let user = &users[0];

    // Delete the user
    let doc = doc! {"id": user.id.as_ref().unwrap().to_string() };
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/delete")
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let body: Document = test::read_body_json(res).await;

    assert_eq!(body.get_str("status").unwrap(), "deleted");

    Ok(())
}
