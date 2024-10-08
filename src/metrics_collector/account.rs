use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_ACCOUNT_CURRENT: IntGaugeVec = register_int_gauge_vec!(
        "op_account_current",
        "Current 1Password account information.",
        &["id", "name", "domain", "type", "state", "created_at"]
    )
    .unwrap();
}

#[derive(Deserialize, Debug)]
struct Account {
    id: String,
    name: String,
    domain: String,
    #[serde(rename = "type")]
    type_: String,
    state: String,
    created_at: String,
}

impl OpMetricsCollector {
    pub(crate) fn read_account(&self) {
        let output = self
            .command_executor
            .exec(vec!["account", "get", "--format", "json"])
            .unwrap();
        let account: Account = serde_json::from_str(&output).unwrap();

        OP_ACCOUNT_CURRENT
            .with_label_values(&[
                &account.id,
                &account.name,
                &account.domain,
                &account.type_,
                &account.state,
                &account.created_at,
            ])
            .set(1);
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_account(metrics_collector: OpMetricsCollector) -> Result<()> {
        // Act
        metrics_collector.read_account();

        // Assert
        assert_eq!(
            OP_ACCOUNT_CURRENT
                .get_metric_with_label_values(&[
                    "??????????????????????????",
                    "**********",
                    "my",
                    "FAMILY",
                    "ACTIVE",
                    "2023-03-19T05:06:27Z",
                ])?
                .get(),
            1
        );

        Ok(())
    }
}
