use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_USER_TOTAL: IntGaugeVec =
        register_int_gauge_vec!("op_user_count_total", "Total number of users.", &[]).unwrap();
}

#[derive(Deserialize, Debug)]
struct User {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) email: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    pub(crate) type_: String,
    #[allow(dead_code)]
    pub(crate) state: String,
}

impl OpMetricsCollector {
    pub(crate) fn read_user(&self) {
        let output = self
            .command_executor
            .exec(vec!["user", "list", "--format", "json"])
            .unwrap();
        let users: Vec<User> = serde_json::from_str(&output).unwrap();

        OP_USER_TOTAL.with_label_values(&[]).set(users.len() as i64);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_user(metrics_collector: OpMetricsCollector) {
        metrics_collector.read_user();

        // Assert
        assert_eq!(
            OP_USER_TOTAL
                .get_metric_with_label_values(&[])
                .unwrap()
                .get(),
            1
        );
    }
}
