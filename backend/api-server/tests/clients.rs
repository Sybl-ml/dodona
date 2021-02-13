use actix_web::web::get;
use actix_web::{middleware, test, App};
use mongodb::bson::doc;

use api_server::routes::clients;

#[macro_use]
mod common;

use common::get_bearer_token;

#[actix_rt::test]
async fn test_get_model_performance() {
    let mut app = api_with! {
        get: "/api/clients/m/{model_id}/performance" => clients::get_model_performance,
    };

    let results: Vec<f64> = vec![0.4, 0.5, 0.6];
    let doc = doc! {"id": common::MODEL_ID};

    let url = format!("/api/clients/m/{}/performance", common::MODEL_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .set_json(&doc)
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let performances: Vec<f64> = test::read_body_json(res).await;

    assert_eq!(performances, results);
}
