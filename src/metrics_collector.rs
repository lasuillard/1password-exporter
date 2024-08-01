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
pub enum Metrics {
    // Metrics that does not consume quota
    Account,
    Group,
    User,
    ServiceAccount,
    BuildInfo,
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
            "group" => Ok(Metrics::Group),
            "user" => Ok(Metrics::User),
            "service-account" => Ok(Metrics::ServiceAccount),
            "build-info" => Ok(Metrics::BuildInfo),
            "document" => Ok(Metrics::Document),
            "item" => Ok(Metrics::Item),
            "vault" => Ok(Metrics::Vault),
            _ => Err(()),
        }
    }
}

pub struct OpMetricsCollector {
    command_executor: Box<dyn CommandExecutor>,
}

impl OpMetricsCollector {
    pub fn new(command_executor: Box<dyn CommandExecutor>) -> Self {
        OpMetricsCollector { command_executor }
    }

    pub fn collect(&self, metrics: Vec<Metrics>) {
        // TODO: Collect all metrics in async manner (use Tokio)
        for metric in metrics {
            match metric {
                Metrics::ServiceAccount => self.collect_serviceaccount(),
                _ => panic!("Metric {:?} not supported", metric),
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
        assert_eq!(Metrics::from_str("group"), Ok(Metrics::Group));
        assert_eq!(Metrics::from_str("user"), Ok(Metrics::User));
        assert_eq!(
            Metrics::from_str("service-account"),
            Ok(Metrics::ServiceAccount)
        );
        assert_eq!(Metrics::from_str("build-info"), Ok(Metrics::BuildInfo));
        assert_eq!(Metrics::from_str("document"), Ok(Metrics::Document));
        assert_eq!(Metrics::from_str("item"), Ok(Metrics::Item));
        assert_eq!(Metrics::from_str("vault"), Ok(Metrics::Vault));
        assert_eq!(Metrics::from_str("unknown"), Err(()));
    }
}
