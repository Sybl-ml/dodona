use actix_web::web::{get, post};
use actix_web::{middleware, test, App};
use mongodb::bson::doc;

use api_server::routes::{clients, users};
use models::users::User;

#[macro_use]
extern crate serde;

#[macro_use]
mod common;

use common::get_bearer_token;

#[actix_rt::test]
async fn users_can_become_clients() {
    let mut app = api_with! {
        get: "/api/users" => users::get,
        post: "/api/clients/register" => clients::register,
    };

    // Register the user as a client
    let doc = doc! { "email": "matthewsmith@email.com", "password": "password" };

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .set_json(&doc)
        .uri("/api/clients/register")
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    // Check that the user is a client
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri("/api/users")
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let user: User = test::read_body_json(res).await;

    assert!(user.client);
}

#[actix_rt::test]
async fn users_cannot_become_clients_twice() {
    let mut app = api_with! {
        post: "/api/clients/register" => clients::register,
    };

    // Attempt to register our client again
    let doc = doc! { "email": "client@sybl.com", "password": "password" };

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_CLIENT_ID)))
        .set_json(&doc)
        .uri("/api/clients/register")
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::CONFLICT, res.status());
}

#[actix_rt::test]
async fn model_performances_can_be_fetched() {
    let mut app = api_with! {
        get: "/api/clients/models/{model_id}/performance" => clients::get_model_performance,
    };

    let results: Vec<f64> = vec![0.4, 0.5, 0.6];
    let doc = doc! {"id": common::MODEL_ID};

    let url = format!("/api/clients/models/{}/performance", common::MODEL_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .set_json(&doc)
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let performances: Vec<f64> = test::read_body_json(res).await;

    assert_eq!(performances, results);
}

#[actix_rt::test]
async fn non_clients_cannot_request_new_private_keys() {
    let mut app = api_with! {
        post: "/api/clients/generatePrivateKey" => clients::generate_private_key,
    };

    let url = "/api/clients/generatePrivateKey";

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::FORBIDDEN, res.status());
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateKeyResponse {
    priv_key: String,
}

#[actix_rt::test]
async fn clients_can_request_new_private_keys() {
    let mut app = api_with! {
        post: "/api/clients/generatePrivateKey" => clients::generate_private_key,
    };

    let url = "/api/clients/generatePrivateKey";

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header((
            "Authorization",
            get_bearer_token(common::PRIVATE_KEY_CLIENT_USER_ID),
        ))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    let status = res.status();
    assert_eq!(actix_web::http::StatusCode::OK, status);

    // This will check the format was correct
    let _private_key: PrivateKeyResponse = test::read_body_json(res).await;
}
