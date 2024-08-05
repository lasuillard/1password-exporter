use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_ITEM_TOTAL: IntGaugeVec =
        register_int_gauge_vec!("op_item_total", "Total number of items.", &[]).unwrap();
}

#[derive(Deserialize, Debug)]
struct Item {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) title: String,
    #[allow(dead_code)]
    pub(crate) version: i32,
    #[allow(dead_code)]
    pub(crate) vault: ItemVault,
    #[allow(dead_code)]
    pub(crate) category: String,
    #[allow(dead_code)]
    pub(crate) last_edited_by: String,
    #[allow(dead_code)]
    pub(crate) created_at: String,
    #[allow(dead_code)]
    pub(crate) updated_at: String,
    #[allow(dead_code)]
    pub(crate) additional_information: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ItemVault {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) name: String,
}

impl OpMetricsCollector {
    pub(crate) fn read_item(&self) {
        let output = self
            .command_executor
            .exec(vec![
                "item",
                "list",
                "--format",
                "json",
                "--include-archive",
            ])
            .unwrap();
        let items: Vec<Item> = serde_json::from_str(&output).unwrap();

        OP_ITEM_TOTAL.with_label_values(&[]).set(items.len() as i64);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::command_executor::MockCommandExecutor;

    #[fixture]
    fn item() -> String {
        r#"
[
  {
    "id": "5zouxmnesgyexgg2sld32g627a",
    "title": "Empty Document",
    "version": 2,
    "vault": {
      "id": "qbdrg7xppiklcy74pf4uw5cmka",
      "name": "Infra: Home Server"
    },
    "category": "DOCUMENT",
    "last_edited_by": "K3MAYGGYRZA2XN2AMQ5ADZJ6VI",
    "created_at": "2024-08-04T04:06:31Z",
    "updated_at": "2024-08-04T04:06:58Z"
  }
]
"#
        .to_string()
    }

    #[rstest]
    fn test_read_item(item: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .with(eq(vec![
                "item",
                "list",
                "--format",
                "json",
                "--include-archive",
            ]))
            .returning(move |_| Ok(item.clone()));
        let collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        collector.read_item();

        // Assert
        assert_eq!(
            OP_ITEM_TOTAL
                .get_metric_with_label_values(&[])
                .unwrap()
                .get(),
            1
        );
    }
}
