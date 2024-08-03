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

/// 1Password service account rate limit information.
#[derive(Debug, PartialEq)]
pub(crate) struct Ratelimit {
    pub(crate) type_: String,
    pub(crate) action: String,
    pub(crate) limit: i32,
    pub(crate) used: i32,
    pub(crate) remaining: i32,
    #[allow(dead_code)]
    pub(crate) reset: String,
}

/// 1Password service account information.
pub(crate) struct Whoami {
    pub(crate) url: String,
    pub(crate) integration_id: String,
    pub(crate) user_type: String,
}

impl OpMetricsCollector {
    fn read_ratelimit(&self) -> Vec<Ratelimit> {
        let output = self
            .command_executor
            .exec(vec!["service-account", "ratelimit"])
            .unwrap();

        let table = crate::utils::parse_table(&output);
        let mut result = Vec::with_capacity(table.len());
        for row in table {
            let ratelimit = Ratelimit {
                type_: row.get("TYPE").unwrap().to_string(),
                action: row.get("ACTION").unwrap().to_string(),
                limit: row.get("LIMIT").unwrap().parse().unwrap(),
                used: row.get("USED").unwrap().parse().unwrap(),
                remaining: row.get("REMAINING").unwrap().parse().unwrap(),
                reset: row.get("RESET").unwrap().to_string(),
            };
            result.push(ratelimit);
        }

        result
    }

    fn read_whoami(&self) -> Whoami {
        let output = self.command_executor.exec(vec!["whoami"]).unwrap();
        let kv = crate::utils::parse_kv(&output);

        Whoami {
            url: kv.get("URL").unwrap().to_string(),
            integration_id: kv.get("Integration ID").unwrap().to_string(),
            user_type: kv.get("User Type").unwrap().to_string(),
        }
    }

    pub(crate) fn collect_serviceaccount(&self) {
        let ratelimit = self.read_ratelimit();
        for rl in ratelimit {
            OP_SERVICEACCOUNT_RATELIMIT_LIMIT
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.limit as i64);
            OP_SERVICEACCOUNT_RATELIMIT_USED
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.used as i64);
            OP_SERVICEACCOUNT_RATELIMIT_REMAINING
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.remaining as i64);
        }

        let whoami = self.read_whoami();
        OP_SERVICEACCOUNT_WHOAMI
            .with_label_values(&[&whoami.url, &whoami.integration_id, &whoami.user_type])
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
        let ratelimits = metrics_collector.read_ratelimit();

        // Assert
        assert_eq!(ratelimits.len(), 3);
        assert_eq!(
            ratelimits[0],
            Ratelimit {
                type_: "token".to_string(),
                action: "write".to_string(),
                limit: 100,
                used: 0,
                remaining: 100,
                reset: "N/A".to_string(),
            }
        );
        assert_eq!(
            ratelimits[1],
            Ratelimit {
                type_: "token".to_string(),
                action: "read".to_string(),
                limit: 1000,
                used: 0,
                remaining: 1000,
                reset: "N/A".to_string(),
            }
        );
        assert_eq!(
            ratelimits[2],
            Ratelimit {
                type_: "account".to_string(),
                action: "read_write".to_string(),
                limit: 1000,
                used: 4,
                remaining: 996,
                reset: "1 hour from now".to_string(),
            }
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
        let whoami = metrics_collector.read_whoami();

        // Assert
        assert_eq!(whoami.url, "https://my.1password.com");
        assert_eq!(whoami.integration_id, "WADYB2CBTFBIFKESZ6AV74PUGE");
        assert_eq!(whoami.user_type, "SERVICE_ACCOUNT");
    }

    #[rstest]
    fn test_collect_serviceaccount(ratelimit: String, whoami: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .with(eq(vec!["service-account", "ratelimit"]))
            .returning(move |_| Ok(ratelimit.clone()));
        command_executor
            .expect_exec()
            .with(eq(vec!["whoami"]))
            .returning(move |_| Ok(whoami.clone()));
        let metrics_collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        metrics_collector.collect_serviceaccount();

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
