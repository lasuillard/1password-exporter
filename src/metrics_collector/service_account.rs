use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_SERVICEACCOUNT_RATELIMIT_USED: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_used",
        "1Password API rate limit used.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_RATELIMIT_LIMIT: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_limit",
        "1Password API rate limit.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_RATELIMIT_REMAINING: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_remaining",
        "1Password API rate limit remaining.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_SERVICEACCOUNT_WHOAMI: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_whoami",
        "1Password service account information.",
        &["url", "integration_id", "user_type"]
    )
    .unwrap();
}

impl OpMetricsCollector {
    pub(crate) fn read_ratelimit(&self) {
        let output = self
            .command_executor
            .exec(vec!["service-account", "ratelimit"])
            .unwrap();

        let table = crate::utils::parse_table(&output);
        for row in table {
            let type_ = row.get("TYPE").unwrap();
            let action = row.get("ACTION").unwrap();
            let limit: i64 = row.get("LIMIT").unwrap().parse().unwrap();
            let used: i64 = row.get("USED").unwrap().parse().unwrap();
            let remaining: i64 = row.get("REMAINING").unwrap().parse().unwrap();

            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .with_label_values(&[type_, action])
                .set(limit as i64);
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .with_label_values(&[type_, action])
                .set(used as i64);
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .with_label_values(&[type_, action])
                .set(remaining as i64);
        }
    }

    pub(crate) fn read_whoami(&self) {
        let output = self.command_executor.exec(vec!["whoami"]).unwrap();
        let kv = crate::utils::parse_kv(&output);

        let url = kv.get("URL").unwrap().to_string();
        let integration_id = kv.get("Integration ID").unwrap().to_string();
        let user_type = kv.get("User Type").unwrap().to_string();

        OP_SERVICEACCOUNT_WHOAMI
            .with_label_values(&[&url, &integration_id, &user_type])
            .set(1);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::command_executor::MockCommandExecutor;

    #[fixture]
    fn ratelimit(#[default("OK")] case: &str) -> String {
        match case {
            "OK" => {
                r#"
TYPE       ACTION        LIMIT    USED    REMAINING    RESET
token      write         100      0       100          N/A
token      read          1000     0       1000         N/A
account    read_write    1000     4       996          1 hour from now
"#
            }
            "Not used" => {
                r#"
TYPE       ACTION        LIMIT    USED    REMAINING    RESET
token      write         100      0       100          N/A
token      read          1000     0       1000         N/A
account    read_write    1000     0       1000         N/A
"#
            }
            _ => panic!("Unsupported case"),
        }
        .to_string()
    }

    #[fixture]
    fn whoami() -> String {
        r#"
URL:               https://my.1password.com
Integration ID:    WADYB2CBTFBIFKESZ6AV74PUGE
User Type:         SERVICE_ACCOUNT
"#
        .to_string()
    }

    #[rstest]
    fn test_read_ratelimit(ratelimit: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .with(eq(vec!["service-account", "ratelimit"]))
            .returning(move |_| Ok(ratelimit.clone()));
        let metrics_collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        metrics_collector.read_ratelimit();

        // Assert
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["token", "write"])
                .unwrap()
                .get(),
            100
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .get_metric_with_label_values(&["token", "write"])
                .unwrap()
                .get(),
            0
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["token", "write"])
                .unwrap()
                .get(),
            100
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["token", "read"])
                .unwrap()
                .get(),
            1000
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .get_metric_with_label_values(&["token", "read"])
                .unwrap()
                .get(),
            0
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["token", "read"])
                .unwrap()
                .get(),
            1000
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["account", "read_write"])
                .unwrap()
                .get(),
            1000
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .get_metric_with_label_values(&["account", "read_write"])
                .unwrap()
                .get(),
            4
        );
        assert_eq!(
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["account", "read_write"])
                .unwrap()
                .get(),
            996
        );
    }

    #[rstest]
    fn test_read_whoami(whoami: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .returning(move |_| Ok(whoami.clone()));
        let metrics_collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        metrics_collector.read_whoami();

        // Assert
        assert_eq!(
            OP_SERVICEACCOUNT_WHOAMI
                .get_metric_with_label_values(&[
                    "https://my.1password.com",
                    "WADYB2CBTFBIFKESZ6AV74PUGE",
                    "SERVICE_ACCOUNT"
                ])
                .unwrap()
                .get(),
            1
        );
    }
}
