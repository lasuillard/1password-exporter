mod command_executor;
mod metrics_scraper;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    crate::server::run_server("0.0.0.0", 9999).await
}
