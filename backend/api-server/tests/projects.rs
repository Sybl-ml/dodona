use actix_web::web::{delete, get, patch, post, put};
use actix_web::{middleware, test, App, Result};
use mongodb::bson::{doc, document::Document};
use serde::{Deserialize, Serialize};

use api_server::routes::projects;
use models::dataset_details::DatasetDetails;
use models::projects::Project;

#[macro_use]
mod common;

use common::get_bearer_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    project: Project,
    details: Option<DatasetDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetResponse {
    dataset: String,
}

#[actix_rt::test]
async fn projects_can_be_fetched_for_a_user() -> Result<()> {
    let mut app = api_with! { get: "/api/projects" => projects::get_user_projects };
    let url = "/api/projects";

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let projects: Vec<Project> = test::read_body_json(res).await;

    println!("{:?}", projects);

    assert_eq!(projects.len(), 2);

    let found = &projects[0];

    assert_eq!("Test Project", found.name);
    assert_eq!("Test Description", found.description);

    Ok(())
}

#[actix_rt::test]
async fn projects_can_be_fetched_by_identifier() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/p/{project_id}" => projects::get_project };

    let formatted = format!("/api/projects/p/{}", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
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
    let mut app = api_with! { get: "/api/projects/p/{project_id}" => projects::get_project };

    let formatted = format!("/api/projects/p/{}", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::DELETE_UID))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::UNAUTHORIZED, res.status());

    Ok(())
}

#[actix_rt::test]
async fn non_existent_projects_are_not_found() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/p/{project_id}" => projects::get_project };

    let formatted = format!("/api/projects/p/{}", common::NON_EXISTENT_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::NOT_FOUND, res.status());

    Ok(())
}

#[actix_rt::test]
async fn projects_cannot_be_found_with_invalid_identifiers() -> Result<()> {
    let mut app = api_with! { get: "/api/projects/p/{project_id}" => projects::get_project };

    let url = "/api/projects/p/invalid";
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
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

    let doc = doc! {"name": "test", "description": "test"};
    let url = "/api/projects/new";

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .header(
            "Authorization",
            get_bearer_token(common::CREATES_PROJECT_UID),
        )
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn datasets_can_be_added_to_projects() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/p/{project_id}/data" => projects::add_data };

    let doc = doc! {"content": "age,sex,location\n22,M,Leamington Spa", "name": "Freddie"};
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn only_one_dataset_can_be_added_to_a_project() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/p/{project_id}/data" => projects::add_data,
        get: "/api/projects/p/{project_id}/data" => projects::get_data,
    };

    let doc = doc! {"content": "age,sex,location\n23,M,Leamington Spa", "name": "Freddie"};
    let url = format!(
        "/api/projects/p/{}/data",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header(
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        )
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let doc = doc! {"content": "age,sex,location\n23,M,Coventry", "name": "Freddie"};
    let url = format!(
        "/api/projects/p/{}/data",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header(
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        )
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let url = format!(
        "/api/projects/p/{}/data",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header(
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        )
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let dataset_response: DatasetResponse = test::read_body_json(res).await;
    assert_eq!(dataset_response.dataset, "age,sex,location\n23,M,Coventry");

    Ok(())
}

#[actix_rt::test]
async fn datasets_cannot_be_added_if_projects_do_not_exist() -> Result<()> {
    let mut app = api_with! { put: "/api/projects/p/{project_id}/data" => projects::add_data };

    let doc = doc! {"content": "age,sex,location\n22,M,Leamington Spa", "name": "Freddie"};
    let url = format!("/api/projects/p/{}/data", common::NON_EXISTENT_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header(
            "Authorization",
            get_bearer_token(common::NON_EXISTENT_USER_ID),
        )
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::NOT_FOUND, res.status());

    Ok(())
}

#[actix_rt::test]
async fn dataset_can_be_taken_from_database() -> Result<()> {
    let mut app = api_with! {
        get: "/api/projects/p/{project_id}/data" => projects::get_data,
        put: "/api/projects/p/{project_id}/data" => projects::add_data,
    };

    let doc = doc! {"content": "age,sex,location\n22,M,Leamington Spa", "name": "Freddie"};
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn overview_of_dataset_can_be_returned() -> Result<()> {
    let mut app = api_with! {
        put: "/api/projects/p/{project_id}/data" => projects::add_data,
        post: "/api/projects/p/{project_id}/overview" => projects::overview,
    };

    let doc = doc! {"content": "age,sex,location\n22,M,Leamington Spa", "name": "Freddie"};
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let url = format!("/api/projects/p/{}/overview", common::MAIN_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}

#[actix_rt::test]
async fn projects_can_be_deleted() -> Result<()> {
    let mut app = api_with! {
        delete: "/api/projects/p/{project_id}" => projects::delete_project,
        post: "/api/projects/p/{project_id}/overview" => projects::overview,
    };

    let formatted = format!("/api/projects/p/{}", common::DELETABLE_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::DELETE)
        .header(
            "Authorization",
            get_bearer_token(common::DELETES_PROJECT_UID),
        )
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let formatted = format!("/api/projects/u/{}", common::DELETES_PROJECT_UID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header(
            "Authorization",
            get_bearer_token(common::DELETES_PROJECT_UID),
        )
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
        patch: "/api/projects/p/{project_id}" => projects::patch_project,
        get: "/api/projects/p/{project_id}" => projects::get_project,
    };

    let formatted = format!("/api/projects/p/{}", common::EDITABLE_PROJECT_ID);
    let doc = doc! {"changes": {"description": "new description"}};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PATCH)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .set_json(&doc)
        .uri(&formatted)
        .to_request();

    let res = test::call_service(&mut app, req).await;
    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let formatted = format!("/api/projects/p/{}", common::EDITABLE_PROJECT_ID);
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::GET)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
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
        put: "/api/projects/p/{project_id}/data" => projects::add_data,
        post: "/api/projects/p/{project_id}/process" => projects::begin_processing,
    };

    let doc = doc! {"content": "age,sex,location\n22,M,Leamington Spa", "name": "Freddie"};
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);

    let req = test::TestRequest::default()
        .method(actix_web::http::Method::PUT)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&url)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    let formatted = format!("/api/projects/p/{}/process", common::MAIN_PROJECT_ID);
    let doc =
        doc! { "timeout": 10 , "predictionType": "classification", "predictionColumn": "name"};
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::POST)
        .header("Authorization", get_bearer_token(common::MAIN_USER_ID))
        .uri(&formatted)
        .set_json(&doc)
        .to_request();

    let res = test::call_service(&mut app, req).await;

    assert_eq!(actix_web::http::StatusCode::OK, res.status());

    Ok(())
}
