use actix_web::web::{get, post};
use actix_web::{middleware, test, App, Result};
use api_server::routes::users;
use mongodb::bson::{doc, document::Document};

use models::users::User;

#[macro_use]
extern crate serde;

#[macro_use]
mod common;

use common::get_bearer_token;

#[derive(Deserialize, Debug)]
struct AuthResponse {
    pub token: String,
}

#[actix_rt::test]
async fn users_can_register() -> Result<()> {
    let mut app = api_with! { post: "/api/users/new" => users::new };

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
    let mut app = api_with! { post: "/api/users/new" => users::new };

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
    assert_eq!(actix_web::http::StatusCode::CONFLICT, res.status());

    Ok(())
}

#[actix_rt::test]
async fn users_can_login() -> Result<()> {
    let mut app = api_with! { post: "/api/users/login" => users::login };

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
    let mut app = api_with! { post: "/api/users/login" => users::login };

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
    let mut app = api_with! { post: "/api/users/login" => users::login };

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
async fn users_can_upload_an_avatar_image() -> Result<()> {
    let mut app = api_with! { post: "/api/users/avatar" => users::new_avatar };

    // Trim here as trailing newlines will cause a 422
    let doc = doc! { "avatar": include_str!("assets/base64_avatar.txt").trim() };

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/avatar")
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn users_can_retrieve_an_avatar_image() -> Result<()> {
    let mut app = api_with! {
        post: "/api/users/avatar" => users::new_avatar,
        get: "/api/users/avatar" => users::get_avatar,
    };

    // Trim here as trailing newlines will cause a 422
    let doc = doc! { "avatar": include_str!("assets/base64_avatar.txt").trim() };

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .uri("/api/users/avatar")
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .uri("/api/users/avatar")
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn filter_finds_given_user_and_no_others() -> Result<()> {
    let mut app = api_with! { post: "/api/users/filter" => users::filter };

    let doc = doc! {"filter": { "email": "matthewsmith@email.com" } };
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
    let mut app = api_with! { post: "/api/users/filter" => users::filter };

    let doc = doc! {"filter": { "email": "nonexistent@email.com" } };
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
    let mut app = api_with! {
        post: "/api/users/filter" => users::filter,
        post: "/api/users/delete" => users::delete,
    };

    // Find the user
    let doc = doc! {"filter": { "email": "delete@me.com" } };
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
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(&user.id.to_string())))
        .uri("/api/users/delete")
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let body: Document = test::read_body_json(res).await;

    assert_eq!(body.get_str("status").unwrap(), "deleted");

    Ok(())
}
