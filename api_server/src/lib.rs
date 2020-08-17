#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate mongodb;
extern crate r2d2;
extern crate r2d2_mongodb;
extern crate serde_json;

use std::collections::HashMap;

use rocket::{get, routes, http::Method, Rocket, Request};
use rocket_contrib::{templates::Template, serve::StaticFiles, json::JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

#[derive(Serialize)]
struct TemplateContext {
    name: String,
    items: Vec<&'static str>
}

#[get("/")]
fn index() -> JsonValue {
    json!({
        "name": "Freddie",
        "age": 22,
    })
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hey from Dodona!"
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path());
    Template::render("error/404", &map)
}

pub fn rocket() -> Rocket {
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["http://localhost:3000", "http://0.0.0.0:3000"]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors().unwrap();

    rocket::ignite()
        .mount("/api", routes![index, hello])
        .mount("/api", StaticFiles::from("static/"))
        .attach(Template::fairing())
        .attach(cors)
        .register(catchers![not_found])
}