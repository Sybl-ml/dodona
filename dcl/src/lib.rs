extern crate tokio;

use anyhow::Result;



pub mod backend;
pub mod frontend;

#[tokio::main]
pub async fn run() -> Result<()> {
    tokio::spawn(async move {
        frontend::run_server().await.unwrap();
    }).await?;
    
    log::info!("(DCL) shutting down...");

    Ok(())
}
