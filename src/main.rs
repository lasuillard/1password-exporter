use clap::Parser;

use crate::metrics_collector::Metrics;

mod command_executor;
mod metrics_collector;
mod server;

/// A simple Prometheus exporter for the 1Password.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Host to bind the server to.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    // TODO: Pick a different port later
    /// Port to bind the server to.
    #[arg(short, long, default_value_t = 9999)]
    port: u16,

    /// Metrics to collect. Only metrics not consuming API rate enabled by default.
    #[arg(short, long, default_values = ["account", "group", "user", "service-account", "build-info"])]
    metrics: Vec<Metrics>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    crate::server::run_server(args.host, args.port, args.metrics).await
}
