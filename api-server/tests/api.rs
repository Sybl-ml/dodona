use serde::Serialize;
use tide::http::{Method, Request, Response, Url};

#[derive(Serialize)]
struct Registration {
    pub email: String,
    pub password: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
}

#[async_std::test]
async fn api() -> tide::Result<()> {
    let app = dodona::build_server().await;

    let url = Url::parse("localhost:/api").unwrap();
    let req = Request::new(Method::Get, url);
    let mut res: Response = app.respond(req).await?;

    let expected = r#"{"name":"Freddie","age":22}"#;

    assert_eq!(expected, res.body_string().await?);

    Ok(())
}

#[async_std::test]
async fn register() -> tide::Result<()> {
    let app = dodona::build_server().await;

    let url = Url::parse("localhost:/api/users/new").unwrap();
    let mut req = Request::new(Method::Post, url);

    let body = Registration {
        email: "alex@email.com".into(),
        password: "password".into(),
        first_name: "Alex".into(),
        last_name: "Jackson".into(),
    };

    req.set_body(serde_json::to_string(&body).unwrap());
    req.set_content_type(tide::http::mime::JSON);

    let mut res: Response = app.respond(req).await?;

    assert_eq!(tide::StatusCode::Ok, res.status());

    let body = res.body_string().await?;

    assert!(body.contains("token"));

    Ok(())
}
