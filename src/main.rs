use clap::Parser;
use simplelog::*;

use crate::metrics_collector::Metrics;

mod command_executor;
mod metrics_collector;
mod server;

#[cfg(test)]
mod testing;

#[cfg(test)]
#[path = "../tests/test_helper.rs"]
pub(crate) mod test_helper;

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

    /// Port to bind the server to.
    #[arg(short, long, default_value_t = 9999)]
    port: u16,

    /// Metrics to collect. Only metrics not consuming API rate enabled by default.
    #[arg(short, long, num_args = 1.., value_delimiter = ',', default_values = ["account", "group", "user", "service-account", "build-info"])]
    metrics: Vec<Metrics>,

    /// Path to 1Password CLI binary.
    #[arg(long, default_value = "op")]
    op_path: String,

    /// Service account token to pass to the 1Password CLI.
    #[arg(long)]
    service_account_token: Option<String>,
}

async fn _main(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    TermLogger::init(
        args.log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    crate::server::run_server(
        args.host,
        args.port,
        args.metrics,
        args.op_path,
        args.service_account_token,
    )
    .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    _main(args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_serving() {
        let port = test_helper::get_random_port();
        let server = tokio::spawn(async move {
            let args = Args {
                log_level: LevelFilter::Debug,
                host: "127.0.0.1".to_string(),
                port,
                op_path: test_helper::MOCK_OP.to_string(),
                service_account_token: Some("ops_blahblah".to_string()),
                metrics: vec![
                    Metrics::Account,
                    Metrics::BuildInfo,
                    Metrics::Document,
                    Metrics::Group,
                    Metrics::Item,
                    Metrics::ServiceAccount,
                    Metrics::User,
                    Metrics::Vault,
                ],
            };
            _main(args).await.unwrap();
        });

        // Wait for the server to start
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let body = reqwest::get(format!("http://localhost:{port}/metrics"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        assert_eq!(body, include_str!(test_dir!("expected_metrics.txt")));

        server.abort();
    }
}
