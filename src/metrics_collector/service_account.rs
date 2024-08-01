use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_RATELIMIT_USED: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_used",
        "1Password API rate limit used.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_RATELIMIT_LIMIT: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_limit",
        "1Password API rate limit.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_RATELIMIT_REMAINING: IntGaugeVec = register_int_gauge_vec!(
        "op_serviceaccount_ratelimit_remaining",
        "1Password API rate limit remaining.",
        &["type", "action"]
    )
    .unwrap();
}

/// 1Password API rate limit data.
///
/// Retrieved from CLI `op service-account ratelimit`.
#[derive(Debug, PartialEq)]
pub struct Ratelimit {
    pub type_: String,
    pub action: String,
    pub limit: i32,
    pub used: i32,
    pub remaining: i32,
    #[allow(dead_code)]
    pub reset: String,
}

impl OpMetricsCollector {
    fn read_ratelimit(&self) -> Vec<Ratelimit> {
        let output = self
            .command_executor
            .exec(vec!["service-account", "ratelimit"])
            .unwrap();
        let lines = output.trim().split('\n').collect::<Vec<&str>>();

        // Iterate skipping the header
        // TODO: Better table deserialization with serde
        let mut result = Vec::new();
        for line in lines.iter().skip(1) {
            let fields = line.split_ascii_whitespace().collect::<Vec<&str>>();
            let ratelimit = Ratelimit {
                type_: fields[0].to_string(),
                action: fields[1].to_string(),
                limit: fields[2].parse().unwrap(),
                used: fields[3].parse().unwrap(),
                remaining: fields[4].parse().unwrap(),
                reset: fields[5..].join(" ").to_string(),
            };
            result.push(ratelimit);
        }

        result
    }

    fn collect_ratelimit(&self) {
        let ratelimit = self.read_ratelimit();
        for rl in ratelimit {
            OP_RATELIMIT_LIMIT
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.limit as i64);
            OP_RATELIMIT_USED
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.used as i64);
            OP_RATELIMIT_REMAINING
                .with_label_values(&[&rl.type_, &rl.action])
                .set(rl.remaining as i64);
        }
    }

    pub fn collect_serviceaccount(&self) {
        self.collect_ratelimit();
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

    #[rstest]
    fn test_read_ratelimit(ratelimit: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
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
    fn test_collect_ratelimit(ratelimit: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .returning(move |_| Ok(ratelimit.clone()));
        let metrics_collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        metrics_collector.collect_ratelimit();

        // Assert
        assert_eq!(
            OP_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["token", "write"])
                .unwrap()
                .get(),
            100
        );
        assert_eq!(
            OP_RATELIMIT_USED
                .get_metric_with_label_values(&["token", "write"])
                .unwrap()
                .get(),
            0
        );
        assert_eq!(
            OP_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["token", "write"])
                .unwrap()
                .get(),
            100
        );
        assert_eq!(
            OP_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["token", "read"])
                .unwrap()
                .get(),
            1000
        );
        assert_eq!(
            OP_RATELIMIT_USED
                .get_metric_with_label_values(&["token", "read"])
                .unwrap()
                .get(),
            0
        );
        assert_eq!(
            OP_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["token", "read"])
                .unwrap()
                .get(),
            1000
        );
        assert_eq!(
            OP_RATELIMIT_LIMIT
                .get_metric_with_label_values(&["account", "read_write"])
                .unwrap()
                .get(),
            1000
        );
        assert_eq!(
            OP_RATELIMIT_USED
                .get_metric_with_label_values(&["account", "read_write"])
                .unwrap()
                .get(),
            4
        );
        assert_eq!(
            OP_RATELIMIT_REMAINING
                .get_metric_with_label_values(&["account", "read_write"])
                .unwrap()
                .get(),
            996
        );
    }
}
