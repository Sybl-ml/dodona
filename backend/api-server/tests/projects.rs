use actix_web::{middleware, test, web, App, Result};
use mongodb::bson::{doc, document::Document};
use serde::{Deserialize, Serialize};

use api_server::routes;
use models::dataset_details::DatasetDetails;
use models::projects::Project;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects")
                    .route(web::get().to(routes::projects::get_user_projects)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects/p/{project_id}")
                    .route(web::get().to(routes::projects::get_project)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects/p/{project_id}")
                    .route(web::get().to(routes::projects::get_project)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects/p/{project_id}")
                    .route(web::get().to(routes::projects::get_project)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects/p/{project_id}")
                    .route(web::get().to(routes::projects::get_project)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects/new").route(web::post().to(routes::projects::new)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .service(
                web::resource("/api/projects/p/{project_id}/data")
                    .route(web::put().to(routes::projects::add_data)),
            ),
    )
    .await;

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
    let state = common::initialise().await;

    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route(
                "/api/projects/p/{project_id}/data",
                web::put().to(routes::projects::add_data),
            )
            .route(
                "/api/projects/p/{project_id}/data",
                web::get().to(routes::projects::get_data),
            ),
    )
    .await;

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
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route(
                "/api/projects/p/{project_id}/data",
                web::put().to(routes::projects::add_data),
            ),
    )
    .await;

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
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route(
                "/api/projects/p/{project_id}/data",
                web::put().to(routes::projects::add_data),
            )
            .route(
                "/api/projects/p/{project_id}/data",
                web::get().to(routes::projects::get_data),
            ),
    )
    .await;

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
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route(
                "/api/projects/p/{project_id}/data",
                web::put().to(routes::projects::add_data),
            )
            .route(
                "/api/projects/p/{project_id}/overview",
                web::post().to(routes::projects::overview),
            ),
    )
    .await;

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
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route(
                "/api/projects/p/{project_id}",
                web::delete().to(routes::projects::delete_project),
            )
            .route(
                "/api/projects/p/{project_id}/overview",
                web::post().to(routes::projects::overview),
            ),
    )
    .await;

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
    let state = common::initialise().await;
    let mut app = test::init_service(
        App::new()
            .wrap(middleware::Logger::default())
            .data(state)
            .route(
                "/api/projects/p/{project_id}",
                web::patch().to(routes::projects::patch_project),
            )
            .route(
                "/api/projects/p/{project_id}",
                web::get().to(routes::projects::get_project),
            ),
    )
    .await;

    let formatted = format!("/api/projects/p/{}", common::EDITABLE_PROJECT_ID);
    let doc = doc! {"description": "new description"};
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
