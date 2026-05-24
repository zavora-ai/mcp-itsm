mod server;
mod store;
mod types;

use rmcp::{ServiceExt, transport::stdio};
use server::ItsmServer;
use store::ItsmStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(ItsmStore::new());
    let server = ItsmServer { store };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
