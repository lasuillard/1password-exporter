use clap::Parser;
use simplelog::*;

use crate::metrics_collector::Metrics;

mod command_executor;
mod metrics_collector;
mod server;
#[cfg(test)]
mod testing;

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
    #[arg(short, long, num_args = 1.., default_values = ["account", "group", "user", "service-account", "build-info"])]
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

    const MOCK_OP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mock_op.bash");

    #[tokio::test]
    async fn test_metrics_serving() {
        let server = tokio::spawn(async {
            let args = Args::parse_from(&[
                "onepassword-exporter",
                "--log-level",
                "DEBUG",
                "--op-path",
                MOCK_OP,
                "--metrics",
                "account",
                "build-info",
                "document",
                "group",
                "item",
                "service-account",
                "user",
                "vault",
                "--service-account-token",
                "ops_blahblah",
            ]);
            _main(args).await.unwrap();
        });

        let body = reqwest::get("http://localhost:9999/metrics")
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
# HELP op_document_count_per_tag Number of documents per tag.
# TYPE op_document_count_per_tag gauge
op_document_count_per_tag{tag="test"} 1
# HELP op_document_count_per_vault Number of documents per vault.
# TYPE op_document_count_per_vault gauge
op_document_count_per_vault{vault="36vhq4xz3r6hnemzadk33evi4a"} 1
# HELP op_document_count_total Total number of documents.
# TYPE op_document_count_total gauge
op_document_count_total 1
# HELP op_exporter_buildinfo Build information of this exporter.
# TYPE op_exporter_buildinfo gauge
op_exporter_buildinfo{version="0.1.0"} 1
# HELP op_group_count_total Total number of groups.
# TYPE op_group_count_total gauge
op_group_count_total 4
# HELP op_item_count_per_category Number of items per category.
# TYPE op_item_count_per_category gauge
op_item_count_per_category{category="DOCUMENT"} 1
op_item_count_per_category{category="LOGIN"} 2
op_item_count_per_category{category="SECURE_NOTE"} 1
op_item_count_per_category{category="SSH_KEY"} 1
# HELP op_item_count_per_tag Number of items per tag.
# TYPE op_item_count_per_tag gauge
op_item_count_per_tag{tag="dev"} 1
op_item_count_per_tag{tag="test"} 4
# HELP op_item_count_per_vault Number of items per vault.
# TYPE op_item_count_per_vault gauge
op_item_count_per_vault{vault="36vhq4xz3r6hnemzadk33evi4a"} 5
# HELP op_item_count_total Total number of items.
# TYPE op_item_count_total gauge
op_item_count_total 5
# HELP op_serviceaccount_ratelimit_limit API rate limit.
# TYPE op_serviceaccount_ratelimit_limit gauge
op_serviceaccount_ratelimit_limit{action="read",type="token"} 1000
op_serviceaccount_ratelimit_limit{action="read_write",type="account"} 1000
op_serviceaccount_ratelimit_limit{action="write",type="token"} 100
# HELP op_serviceaccount_ratelimit_remaining API rate limit remaining.
# TYPE op_serviceaccount_ratelimit_remaining gauge
op_serviceaccount_ratelimit_remaining{action="read",type="token"} 999
op_serviceaccount_ratelimit_remaining{action="read_write",type="account"} 999
op_serviceaccount_ratelimit_remaining{action="write",type="token"} 100
# HELP op_serviceaccount_ratelimit_reset_seconds API rate limit remaining.
# TYPE op_serviceaccount_ratelimit_reset_seconds gauge
op_serviceaccount_ratelimit_reset_seconds{action="read",type="token"} 308
op_serviceaccount_ratelimit_reset_seconds{action="read_write",type="account"} 83108
op_serviceaccount_ratelimit_reset_seconds{action="write",type="token"} 0
# HELP op_serviceaccount_ratelimit_used API rate limit used.
# TYPE op_serviceaccount_ratelimit_used gauge
op_serviceaccount_ratelimit_used{action="read",type="token"} 1
op_serviceaccount_ratelimit_used{action="read_write",type="account"} 1
op_serviceaccount_ratelimit_used{action="write",type="token"} 0
# HELP op_serviceaccount_whoami Current service account information.
# TYPE op_serviceaccount_whoami gauge
op_serviceaccount_whoami{account_uuid="++++++++++++++++++++++++++",url="https://my.1password.com",user_type="SERVICE_ACCOUNT",user_uuid="!!!!!!!!!!!!!!!!!!!!!!!!!!"} 1
# HELP op_user_count_total Total number of users.
# TYPE op_user_count_total gauge
op_user_count_total 1
# HELP op_vault_count_total Total number of vaults.
# TYPE op_vault_count_total gauge
op_vault_count_total 1
"#.strip_prefix("\n").unwrap()  // NOTE: Added preceding newline to make it readable
        );

        server.abort();
    }
}
