use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_VAULT_TOTAL: IntGaugeVec =
        register_int_gauge_vec!("op_vault_count_total", "Total number of vaults.", &[]).unwrap();
}

#[derive(Deserialize, Debug)]
struct Vault {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) content_version: i32,
}

impl OpMetricsCollector {
    pub(crate) fn read_vault(&self) {
        let output = self
            .command_executor
            .exec(vec!["vault", "list", "--format", "json"])
            .unwrap();
        let vaults: Vec<Vault> = serde_json::from_str(&output).unwrap();

        OP_VAULT_TOTAL
            .with_label_values(&[])
            .set(vaults.len() as i64);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::command_executor::MockCommandExecutor;

    #[fixture]
    fn vault() -> String {
        r#"
[
  {
    "id": "qbdrg7xppiklcy74pf4uw5cmka",
    "name": "Infra: Home Server",
    "content_version": 66
  }
]
"#
        .to_string()
    }

    #[rstest]
    fn test_read_vault(vault: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .with(eq(vec!["vault", "list", "--format", "json"]))
            .returning(move |_| Ok(vault.clone()));
        let collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        collector.read_vault();

        // Assert
        assert_eq!(
            OP_VAULT_TOTAL
                .get_metric_with_label_values(&[])
                .unwrap()
                .get(),
            1
        );
    }
}
