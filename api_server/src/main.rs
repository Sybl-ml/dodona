#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

#[cfg(test)] mod tests;

use std::collections::HashMap;

use rocket::Request;
// use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket::http::Method;
use rocket::{get, routes};
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Error, CorsOptions};

#[derive(Serialize)]
struct TemplateContext {
    name: String,
    items: Vec<&'static str>
}

#[get("/")]
fn index() -> JsonValue {
    // let name = "Person".to_string();
    // let context = TemplateContext { name, items: vec!["One", "Two", "Three"] };
    // Template::render("index", &context)
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

fn main() -> Result<(), Error> {
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["http://localhost:3000"]),
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors()?;

    rocket::ignite()
        .mount("/api", routes![index, hello])
        .mount("/api", StaticFiles::from("static/"))
        .attach(Template::fairing())
        .attach(cors)
        .register(catchers![not_found]).launch();

    Ok(())
}