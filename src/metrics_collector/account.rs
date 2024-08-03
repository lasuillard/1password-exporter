use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_ACCOUNT_CURRENT: IntGaugeVec = register_int_gauge_vec!(
        "op_account_current",
        "Current 1Password account information.",
        &["id", "name", "domain", "type", "state"]
    )
    .unwrap();
}

impl OpMetricsCollector {
    pub(crate) fn read_account(&self) {
        let output = self.command_executor.exec(vec!["account", "get"]).unwrap();
        let kv = crate::utils::parse_kv(&output);

        let id = kv.get("ID").unwrap();
        let name = kv.get("Name").unwrap();
        let domain = kv.get("Domain").unwrap();
        let type_ = kv.get("Type").unwrap();
        let state = kv.get("State").unwrap();

        OP_ACCOUNT_CURRENT
            .with_label_values(&[id, name, domain, type_, state])
            .set(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_executor::MockCommandExecutor;

    #[test]
    fn test_read_account() {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .with(eq(vec!["account", "get"]))
            .returning(|_| {
                Ok(r#"
ID:         ??????????????????????????
Name:       **********
Domain:     my
Type:       FAMILY
State:      ACTIVE
Created:    1 year ago
"#
                .to_string())
            });
        let collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        collector.read_account();

        // Assert
        assert_eq!(
            OP_ACCOUNT_CURRENT
                .get_metric_with_label_values(&[
                    "??????????????????????????",
                    "**********",
                    "my",
                    "FAMILY",
                    "ACTIVE",
                ])
                .unwrap()
                .get(),
            1
        );
    }
}
