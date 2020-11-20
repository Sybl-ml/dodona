use serde::{Deserialize, Serialize};
use tide::http::{Request, Response, Url};

use models::dataset_details::DatasetDetails;
use models::projects::Project;

mod common;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    project: Project,
    details: DatasetDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetResponse {
    dataset: String,
}

#[async_std::test]
async fn projects_can_be_fetched_for_a_user() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let formatted = format!("localhost:/api/projects/u/{}", common::MAIN_USER_ID);
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let projects: Vec<Project> = res.body_json().await?;

    assert_eq!(projects.len(), 2);

    let found = &projects[0];

    assert_eq!("Test Project", found.name);
    assert_eq!("Test Description", found.description);
    assert_eq!(0, found.date_created.timestamp_millis());

    Ok(())
}

#[async_std::test]
async fn projects_must_be_tied_to_a_user() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let formatted = format!("localhost:/api/projects/u/{}", common::NON_EXISTENT_USER_ID);
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::NotFound, res.status());

    Ok(())
}

#[async_std::test]
async fn projects_cannot_be_found_for_invalid_user_ids() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let url = Url::parse("localhost:/api/projects/u/invalid").unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::NotFound, res.status());

    Ok(())
}

#[async_std::test]
async fn projects_can_be_fetched_by_identifier() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let formatted = format!("localhost:/api/projects/p/{}", common::MAIN_PROJECT_ID);
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let project_response: ProjectResponse = res.body_json().await?;

    assert_eq!("Test Project", project_response.project.name);
    assert_eq!("Test Description", project_response.project.description);
    assert_eq!(0, project_response.project.date_created.timestamp_millis());

    Ok(())
}

#[async_std::test]
async fn non_existent_projects_are_not_found() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let formatted = format!(
        "localhost:/api/projects/p/{}",
        common::NON_EXISTENT_PROJECT_ID
    );
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::NotFound, res.status());

    Ok(())
}

#[async_std::test]
async fn projects_cannot_be_found_with_invalid_identifiers() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let url = Url::parse("localhost:/api/projects/p/invalid").unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::UnprocessableEntity, res.status());

    Ok(())
}

#[async_std::test]
async fn projects_cannot_be_created_for_non_existent_users() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let url_str = format!(
        "localhost:/api/projects/u/{}/new",
        common::USERLESS_PROJECT_ID
    );
    let url = Url::parse(&url_str).unwrap();
    let req = Request::new(tide::http::Method::Post, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::UnprocessableEntity, res.status());

    Ok(())
}

#[async_std::test]
async fn projects_can_be_created() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"name": "test", "description": "test"}"#;
    let url = format!("/api/projects/u/{}/new", common::CREATES_PROJECT_UID);
    let req = common::build_json_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    Ok(())
}

#[async_std::test]
async fn datasets_can_be_added_to_projects() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"content": "age,sex,location\n22,M,Leamington Spa"}"#;
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let req = common::build_json_put_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    Ok(())
}

#[async_std::test]
async fn only_one_dataset_can_be_added_to_a_project() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"content": "age,sex,location\n22,M,Leamington Spa"}"#;
    let url = format!(
        "/api/projects/p/{}/data",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );
    let req = common::build_json_put_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    let body = r#"{"content": "age,sex,location\n23,M,Leamington Spa"}"#;
    let url = format!(
        "/api/projects/p/{}/data",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );
    let req = common::build_json_put_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    let url = format!(
        "localhost:/api/projects/p/{}/data",
        common::OVERWRITTEN_DATA_PROJECT_ID
    );
    let url = Url::parse(&url).unwrap();
    let req = Request::new(tide::http::Method::Get, url);
    let mut res: Response = app.respond(req).await?;

    let dataset_response: DatasetResponse = res.body_json().await?;

    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(
        dataset_response.dataset,
        "age,sex,location\n23,M,Leamington Spa"
    );

    Ok(())
}

#[async_std::test]
async fn datasets_cannot_be_added_if_projects_do_not_exist() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"content": "age,sex,location\n22,M,Leamington Spa"}"#;
    let url = format!("/api/projects/p/{}/data", common::NON_EXISTENT_PROJECT_ID);
    let req = common::build_json_put_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::NotFound, res.status());

    Ok(())
}

#[async_std::test]
async fn dataset_can_be_taken_from_database() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"content": "age,sex,location\n22,M,Leamington Spa"}"#;
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let req = common::build_json_put_request(&url, body);
    let res: Response = app.respond(req).await?;

    assert_eq!(tide::StatusCode::Ok, res.status());

    let url = format!("localhost:/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let url = Url::parse(&url).unwrap();
    let req = Request::new(tide::http::Method::Get, url);
    let res: Response = app.respond(req).await?;

    assert_eq!(tide::StatusCode::Ok, res.status());

    Ok(())
}

#[async_std::test]
async fn overview_of_dataset_can_be_returned() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let body = r#"{"content": "age,sex,location\n22,M,Leamington Spa"}"#;
    let url = format!("/api/projects/p/{}/data", common::MAIN_PROJECT_ID);
    let req = common::build_json_put_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    let body = r#"{}"#;
    let url = format!("/api/projects/p/{}/overview", common::MAIN_PROJECT_ID);
    let req = common::build_json_request(&url, body);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    Ok(())
}

#[async_std::test]
async fn projects_can_be_deleted() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let formatted = format!("localhost:/api/projects/p/{}", common::DELETABLE_PROJECT_ID);
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Delete, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    let formatted = format!("localhost:/api/projects/u/{}", common::DELETES_PROJECT_UID);
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Get, url);
    let mut res: Response = app.respond(req).await?;

    let projects: Vec<Project> = res.body_json().await?;
    assert_eq!(projects.len(), 0);

    Ok(())
}

#[async_std::test]
async fn projects_can_be_edited() -> tide::Result<()> {
    common::initialise();
    let app = dodona::build_server().await;

    let formatted = format!("localhost:/api/projects/p/{}", common::EDITABLE_PROJECT_ID);
    let url = tide::http::Url::parse(&formatted).unwrap();
    let mut req = tide::http::Request::new(tide::http::Method::Patch, url);
    let body = r#"{"description": "new description"}"#;
    req.set_body(body);
    req.set_content_type(tide::http::mime::JSON);

    let res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());

    /*
    let formatted = format!("localhost:/api/projects/p/{}", common::EDITABLE_PROJECT_ID);
    let url = Url::parse(&formatted).unwrap();
    let req = Request::new(tide::http::Method::Get, url);

    let mut res: Response = app.respond(req).await?;
    assert_eq!(tide::StatusCode::Ok, res.status());
    assert_eq!(Some(tide::http::mime::JSON), res.content_type());

    let project_response: ProjectResponse = res.body_json().await?;

    assert_eq!("new description", project_response.project.description);
    */
    Ok(())
}
