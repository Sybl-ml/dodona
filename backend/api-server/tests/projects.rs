use actix_web::web::{delete, get, patch, post, put};
use actix_web::{middleware, test, App, Result};
use mongodb::bson::{doc, document::Document};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use api_server::routes::projects;
use models::dataset_details::DatasetDetails;
use models::projects::Project;
use models::{dataset_analysis::DatasetAnalysis, jobs::Job};

#[macro_use]
mod common;

use common::get_bearer_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    project: Project,
    details: Option<DatasetDetails>,
    analysis: Option<DatasetAnalysis>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetResponse {
    dataset: String,
}

static ASL_CSV: &str = include_str!("assets/asl.csv");
static DASL_CSV: &str = include_str!("assets/dasl.csv");
static ONLY_TRAIN_DATASET: &str = include_str!("assets/only_train_dataset.csv");
static ONLY_PREDICT_DATASET: &str = include_str!("assets/only_predict_dataset.csv");

#[actix_rt::test]
async fn projects_can_be_fetched_for_a_user() -> Result<()> {
    let mut app = api_with! { get: "/api/projects" => projects::get_user_projects };
    let url = "/api/projects";

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let projects: Vec<ProjectResponse> = test::read_body_json(res).await;

    assert_eq!(projects.len(), 4);

    let found = &projects[0];

    assert_eq!("Test Project", found.project.name);
    assert_eq!("Test Description", found.project.description);

    Ok(())
}

#[actix_rt::test]
async fn projects_can_be_fetched_by_identifier() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/{project_id}" => projects::get_project };

    let formatted = format!("/api/projects/{}", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());
    assert_eq!(
        "application/json",
        res.headers().get("content-type").unwrap()
    );

    let project_response: Document = test::read_body_json(res).await;

    assert_eq!(
        "Test Project",
        project_response
            .get_document("project")
            .unwrap()
            .get_str("name")
            .unwrap()
    );
    assert_eq!(
        "Test Description",
        project_response
            .get_document("project")
            .unwrap()
            .get_str("description")
            .unwrap()
    );

    Ok(())
}

#[actix_rt::test]
async fn projects_cannot_be_fetched_by_users_who_do_not_own_it() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/{project_id}" => projects::get_project };

    let formatted = format!("/api/projects/{}", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::DELETE_UID)))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::FORBIDDEN, res.status());

    Ok(())
}

#[actix_rt::test]
async fn non_existent_projects_are_not_found() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/{project_id}" => projects::get_project };

    let formatted = format!("/api/projects/{}", common::NON_EXISTENT_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::NOT_FOUND, res.status());

    Ok(())
}

#[actix_rt::test]
async fn projects_cannot_be_found_with_invalid_identifiers() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/{project_id}" => projects::get_project };

    let url = "/api/projects/invalid";
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(
        actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
        res.status()
    );

    Ok(())
}

#[actix_rt::test]
async fn projects_can_be_created() -> Result<()> {
    let mut app = api_with! { post: "/api/projects/new" => projects::new };

    let doc = doc! {"name": "test", "description": "test", "tags": ["test"]};
    let url = "/api/projects/new";

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header((
            "Authorization",
            get_bearer_token(common::CREATES_PROJECT_UID),
        ))
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn datasets_can_be_added_to_projects() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split };

    let url = format!("/api/projects/{}/upload_and_split", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn datasets_sizes_are_calculated() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split,
        get: "/api/projects/{project_id}" => projects::get_project,
    };

    let url = format!("/api/projects/{}/upload_and_split", common::DD_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    sleep(Duration::from_millis(500)).await;

    let formatted = format!("/api/projects/{}", common::DD_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let project_response: ProjectResponse = test::read_body_json(res).await;
    let dataset_details: DatasetDetails = project_response.details.unwrap();

    assert_eq!(2, dataset_details.train_size);
    assert_eq!(1, dataset_details.predict_size);

    Ok(())
}

#[actix_rt::test]
async fn only_one_dataset_can_be_added_to_a_project() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split,
        get: "/api/projects/{project_id}/data/{dataset_type}" => projects::get_dataset,
    };

    let url = format!(
        "/api/projects/{}/upload_and_split",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header((
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        ))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let url = format!(
        "/api/projects/{}/upload_and_split",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );
    let modified = ASL_CSV.replace("23", "24");

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header((
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        ))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(modified)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let url = format!(
        "/api/projects/{}/data/train",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header((
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        ))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    let status = res.status();
    let body = test::read_body(res).await;

    assert_eq!(actix_web::http::StatusCode::OK, status);
    assert_eq!(
        std::str::from_utf8(&body).unwrap(),
        "age,sex,location\r\n24,M,Leamington Spa\r"
    );

    Ok(())
}

#[actix_rt::test]
async fn datasets_cannot_be_added_if_projects_do_not_exist() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split };

    let url = format!(
        "/api/projects/{}/upload_and_split",
        common::NON_EXISTENT_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header((
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        ))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::NOT_FOUND, res.status());

    Ok(())
}

#[actix_rt::test]
async fn dataset_can_be_taken_from_database() -> Result<()> {
    let mut app = api_with! {
        get: "/api/projects/{project_id}/data/{dataset_type}" => projects::get_dataset,
        put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split,
    };

    let url = format!("/api/projects/{}/upload_and_split", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    sleep(Duration::from_millis(600)).await;

    let url = format!("/api/projects/{}/data/train", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn overview_of_dataset_can_be_returned() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split,
        post: "/api/projects/{project_id}/overview" => projects::overview,
    };

    let url = format!("/api/projects/{}/upload_and_split", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    sleep(Duration::from_millis(600)).await;

    let url = format!("/api/projects/{}/overview", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn projects_can_be_deleted() -> Result<()> {
    let mut app = api_with! {
        delete: "/api/projects/{project_id}" => projects::delete_project,
        post: "/api/projects/{project_id}/overview" => projects::overview,
    };

    let formatted = format!("/api/projects/{}", common::DELETABLE_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::DELETE)
        .insert_header((
            "Authorization",
            get_bearer_token(common::DELETES_PROJECT_UID),
        ))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let formatted = format!("/api/projects/u/{}", common::DELETES_PROJECT_UID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header((
            "Authorization",
            get_bearer_token(common::DELETES_PROJECT_UID),
        ))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    let projects = test::read_body(res).await;
    assert_eq!(projects, actix_web::web::Bytes::from(""));

    Ok(())
}

#[actix_rt::test]
async fn projects_can_be_edited() -> Result<()> {
    let mut app = api_with! {
        patch: "/api/projects/{project_id}" => projects::patch_project,
        get: "/api/projects/{project_id}" => projects::get_project,
    };

    let formatted = format!("/api/projects/{}", common::EDITABLE_PROJECT_ID);
    let doc = doc! {"changes": {"description": "new description"}};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PATCH)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .set_json(&doc)
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let formatted = format!("/api/projects/{}", common::EDITABLE_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let project_response: ProjectResponse = test::read_body_json(res).await;

    assert_eq!("new description", project_response.project.description);
    Ok(())
}

#[actix_rt::test]
async fn job_configs_can_have_integer_timeouts_in_json() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split,
        post: "/api/projects/{project_id}/process" => projects::begin_processing,
    };

    let url = format!("/api/projects/{}/upload_and_split", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let formatted = format!("/api/projects/{}/process", common::MAIN_PROJECT_ID);
    let doc = doc! { "nodeComputationTime": 10, "clusterSize": 2, "predictionType": "classification", "predictionColumn": "name"};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&formatted)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn train_and_predict_can_be_added_to_projects() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/{project_id}/upload_train_and_predict" => projects::upload_train_and_predict };

    let url = format!(
        "/api/projects/{}/upload_train_and_predict",
        common::MAIN_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(DASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());
    Ok(())
}

#[actix_rt::test]
async fn only_uploading_training_data_fails() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/{project_id}/upload_train_and_predict" => projects::upload_train_and_predict };

    let url = format!(
        "/api/projects/{}/upload_train_and_predict",
        common::MAIN_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ONLY_TRAIN_DATASET)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(
        actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
        res.status()
    );

    Ok(())
}

#[actix_rt::test]
async fn only_uploading_prediction_data_fails() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/{project_id}/upload_train_and_predict" => projects::upload_train_and_predict };

    let url = format!(
        "/api/projects/{}/upload_train_and_predict",
        common::MAIN_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ONLY_PREDICT_DATASET)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(
        actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
        res.status()
    );

    Ok(())
}

#[actix_rt::test]
async fn users_cannot_submit_jobs_with_insufficient_funds() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/{project_id}/upload_and_split" => projects::upload_and_split,
        post: "/api/projects/{project_id}/process" => projects::begin_processing,
    };

    let url = format!("/api/projects/{}/upload_and_split", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
        .uri(&url)
        .set_payload(ASL_CSV)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    sleep(Duration::from_millis(600)).await;

    let formatted = format!("/api/projects/{}/process", common::MAIN_PROJECT_ID);
    let doc = doc! { "nodeComputationTime": 10, "clusterSize": 200000, "predictionType": "classification", "predictionColumn": "name" };

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&formatted)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::PAYMENT_REQUIRED, res.status());

    Ok(())
}

#[actix_rt::test]
async fn recent_jobs_can_be_found() -> Result<()> {
    let mut app = api_with! {
        get: "/api/projects/{project_id}/job" => projects::currently_running_job,
    };

    let url = format!("/api/projects/{}/job", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentJobResponse {
    job: Option<Job>,
}

#[actix_rt::test]
async fn most_recent_job_is_returned() -> Result<()> {
    let mut app = api_with! {
        get: "/api/projects/{project_id}/job" => projects::currently_running_job,
    };

    let url = format!("/api/projects/{}/job", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .insert_header(("Authorization", get_bearer_token(common::MAIN_USER_ID)))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    // Verify the response was correct
    let body: RecentJobResponse = test::read_body_json(res).await;
    assert!(body.job.is_some());

    Ok(())
}
