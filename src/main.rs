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
        let handle = tokio::spawn(async {
            let args = Args::parse_from(&[
                "onepassword-exporter",
                "--log-level",
                "DEBUG",
                "--op-path",
                "/workspaces/1password-exporter/tests/mock_op.bash",
                "--metrics",
                "account",
            ]);
            _main(args).await.unwrap();
        });

        let body = reqwest::get("http://localhost:9999")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        assert_eq!(
            body,
            r#"
# HELP op_account_current Current 1Password account information.
# TYPE op_account_current gauge
op_account_current{created_at="2023-03-19T05:06:27Z",domain="my",id="??????????????????????????",name="**********",state="ACTIVE",type="FAMILY"} 1
"#.strip_prefix("\n").unwrap()  // NOTE: Added preceding newline to make it readable
        );

        handle.abort();
    }
}
