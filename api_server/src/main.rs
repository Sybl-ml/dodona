use async_std::sync::Arc;
use mongodb::Client;
use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use dotenv::dotenv;
use std::env;
use mongodb::options::{StreamAddress, Credential, ClientOptions};

use dodona::routes;
use dodona::State;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    
    dotenv().ok();
    let mongo_addr = env::var("MONGO_ADDR").expect("MONGO_ADDR must be set");
    let mongo_port = env::var("MONGO_PORT").expect("MONGO_PORT must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME env var must be set");
    let usr_name = env::var("USR_NAME").expect("USR_NAME must be set");
    let usr_pwd = env::var("USR_PWD").expect("USR_PWD must be set");
    let credential = Credential::builder()
                        .username(Some(String::from(&usr_name)))
                        .password(Some(String::from(&usr_pwd))).build();
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");

    let options = ClientOptions::builder()
                                .hosts(vec![
                                    StreamAddress {
                                        hostname: mongo_addr.into(),
                                        port: Some(mongo_port.parse::<u16>().unwrap()),
                                    }
                                ])
                                .app_name(Some(String::from(app_name)))
                                // .credential(Some(credential))
                                .build();
    

    // // Configuring DB connection
    // let mut client_options = match ClientOptions::parse("mongodb://localhost:27017").await {
    //     Ok(c) => c,
    //     Err(e) => panic!("Client Options Failed: {}", e)
    // };

    let client = match Client::with_options(options) {
        Ok(c) => c,
        Err(e) => panic!("Client Creation Failed: {}", e)
    };


    let engine = State {
        client: Arc::new(client),
        db_name: Arc::new(String::from(db_name))
    };

    let mut app = tide::with_state(engine);

    // Setting up routes
    let mut core_api = app.at("/api");
    core_api.at("/").get(routes::index);
    core_api.at("/hello").get(routes::hello);

    let mut user_api = app.at("/api/users");
    user_api.at("/").get(routes::users::show);

    
    // CORS
    let cors = CorsMiddleware::new()
    .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
    .allow_origin(Origin::from("*"))
    .allow_credentials(false);

    app.with(cors);

    // Serving App
    tide::log::start();
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.listen("0.0.0.0:3001").await?;
    Ok(())
}