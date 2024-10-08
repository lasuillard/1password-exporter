use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_SERVICEACCOUNT_RATELIMIT_USED: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_used",
        "API rate limit used.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_RATELIMIT_LIMIT: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_limit",
        "API rate limit.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_RATELIMIT_REMAINING: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_remaining",
        "API rate limit remaining.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_RATELIMIT_RESET: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_reset_seconds",
        "API rate limit remaining.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_WHOAMI: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_whoami",
        "Current service account information.",
        &["url", "user_uuid", "account_uuid", "user_type"]
    )
    .unwrap();
}

#[derive(Deserialize, Debug)]
struct Ratelimit {
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) action: String,
    pub(crate) limit: i64,
    pub(crate) used: i64,
    pub(crate) remaining: i64,
    pub(crate) reset: i64, // Remaining seconds until the rate limit resets.
}

#[derive(Deserialize, Debug)]
struct Whoami {
    pub(crate) url: String,
    pub(crate) user_uuid: String,
    pub(crate) account_uuid: String,
    pub(crate) user_type: String,
}

impl OpMetricsCollector {
    pub(crate) fn read_ratelimit(&self) {
        let output = self
            .command_executor
            .exec(vec!["service-account", "ratelimit", "--format", "json"])
            .unwrap();
        let ratelimit: Vec<Ratelimit> = serde_json::from_str(&output).unwrap();

        for rl in ratelimit {
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.limit);
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.used);
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.remaining);
            OP_SERVICEACCOUNT_RATELIMIT_RESET
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.reset);
        }
    }

    pub(crate) fn read_whoami(&self) {
        let output = self
            .command_executor
            .exec(vec!["whoami", "--format", "json"])
            .unwrap();
        let whoami: Whoami = serde_json::from_str(&output).unwrap();

        OP_SERVICEACCOUNT_WHOAMI
            .with_label_values(&[
                &whoami.url,
                &whoami.user_uuid,
                &whoami.account_uuid,
                &whoami.user_type,
            ])
            .set(1);
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_ratelimit(metrics_collector: OpMetricsCollector) -> Result<()> {
        metrics_collector.read_ratelimit();

        // Assert
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["token", "write"])?
                .get(),
            100
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .get_metric_with_label_values(&["token", "write"])?
                .get(),
            0
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["token", "write"])?
                .get(),
            100
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_RESET
                .get_metric_with_label_values(&["token", "write"])?
                .get(),
            0
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["token", "read"])?
                .get(),
            1000
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .get_metric_with_label_values(&["token", "read"])?
                .get(),
            1
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["token", "read"])?
                .get(),
            999
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_RESET
                .get_metric_with_label_values(&["token", "read"])?
                .get(),
            308
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["account", "read_write"])?
                .get(),
            1000
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .get_metric_with_label_values(&["account", "read_write"])?
                .get(),
            1
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["account", "read_write"])?
                .get(),
            999
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_RESET
                .get_metric_with_label_values(&["account", "read_write"])?
                .get(),
            83108
        );

        Ok(())
    }

    #[rstest]
    fn test_read_whoami(metrics_collector: OpMetricsCollector) -> Result<()> {
        metrics_collector.read_whoami();

        // Assert
        assert_eq!(
            OP_SERVICEACCOUNT_WHOAMI
                .get_metric_with_label_values(&[
                    "https://my.1password.com",
                    "!!!!!!!!!!!!!!!!!!!!!!!!!!",
                    "++++++++++++++++++++++++++",
                    "SERVICE_ACCOUNT"
                ])?
                .get(),
            1
        );

        Ok(())
    }
}
