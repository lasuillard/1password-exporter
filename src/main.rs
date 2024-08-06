use clap::Parser;
use simplelog::*;

use crate::metrics_collector::Metrics;

mod command_executor;
mod metrics_collector;
mod server;

/// A simple Prometheus exporter for the 1Password.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Log level.
    #[arg(long, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter,

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

    /// Path to 1Password CLI binary.
    #[arg(long, default_value = "op")]
    op_path: String,

    /// Service account token to pass to the 1Password CLI.
    #[arg(long)]
    service_account_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    TermLogger::init(
        args.log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    log::debug!("Logger initialized.");

    crate::server::run_server(
        args.host,
        args.port,
        args.metrics,
        args.op_path,
        args.service_account_token,
    )
    .await
}
