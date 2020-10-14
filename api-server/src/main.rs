#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().unwrap();
    let app = dodona::build_server().await;

    tide::log::start();
    app.listen("0.0.0.0:3001").await?;

    Ok(())
}
