use std::str::FromStr;

use crate::command_executor::CommandExecutor;

mod account;
mod build_info;
mod document;
mod group;
mod item;
mod service_account;
mod user;
mod vault;

#[derive(Copy, Clone, Debug, PartialEq, clap::ValueEnum)]
pub(crate) enum Metrics {
    // Metrics that does not consume quota
    Account,
    BuildInfo,
    Group,
    ServiceAccount,
    User,
    // Metrics that consume quota by read
    Document,
    Item,
    Vault,
}

impl FromStr for Metrics {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "account" => Ok(Metrics::Account),
            "build-info" => Ok(Metrics::BuildInfo),
            "document" => Ok(Metrics::Document),
            "group" => Ok(Metrics::Group),
            "item" => Ok(Metrics::Item),
            "service-account" => Ok(Metrics::ServiceAccount),
            "user" => Ok(Metrics::User),
            "vault" => Ok(Metrics::Vault),
            _ => Err(()),
        }
    }
}

pub(crate) struct OpMetricsCollector {
    command_executor: Box<dyn CommandExecutor>,
}

impl OpMetricsCollector {
    pub(crate) fn new(command_executor: Box<dyn CommandExecutor>) -> Self {
        OpMetricsCollector { command_executor }
    }

    pub(crate) fn collect(&self, metrics: Vec<Metrics>) {
        // TODO: Collect all metrics in async manner (use Tokio)
        for metric in metrics {
            match metric {
                Metrics::Account => self.read_account(),
                Metrics::BuildInfo => self.read_buildinfo(),
                Metrics::Document => self.read_document(),
                Metrics::Group => self.read_group(),
                Metrics::Item => self.read_item(),
                Metrics::ServiceAccount => {
                    self.read_ratelimit();
                    self.read_whoami();
                }
                Metrics::User => self.read_user(),
                Metrics::Vault => self.read_vault(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_from_str() {
        assert_eq!(Metrics::from_str("account"), Ok(Metrics::Account));
        assert_eq!(Metrics::from_str("build-info"), Ok(Metrics::BuildInfo));
        assert_eq!(Metrics::from_str("document"), Ok(Metrics::Document));
        assert_eq!(Metrics::from_str("group"), Ok(Metrics::Group));
        assert_eq!(Metrics::from_str("item"), Ok(Metrics::Item));
        assert_eq!(
            Metrics::from_str("service-account"),
            Ok(Metrics::ServiceAccount)
        );
        assert_eq!(Metrics::from_str("user"), Ok(Metrics::User));
        assert_eq!(Metrics::from_str("vault"), Ok(Metrics::Vault));

        assert_eq!(Metrics::from_str("unknown"), Err(()));
    }
}
