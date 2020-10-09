#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let app = dodona::build_server().await;

    tide::log::start();
    app.listen("0.0.0.0:3001").await?;

    Ok(())
}
