use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_GROUP_COUNT_TOTAL: IntGaugeVec =
        register_int_gauge_vec!("op_group_count_total", "Total number of groups.", &[]).unwrap();
}

#[derive(Deserialize, Debug)]
struct Group {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) description: String,
    #[allow(dead_code)]
    pub(crate) state: String,
    #[allow(dead_code)]
    pub(crate) created_at: String,
}

impl OpMetricsCollector {
    pub(crate) fn read_group(&self) {
        let output = self
            .command_executor
            .exec(vec!["group", "list", "--format", "json"])
            .unwrap();
        let groups: Vec<Group> = serde_json::from_str(&output).unwrap();

        OP_GROUP_COUNT_TOTAL
            .with_label_values(&[])
            .set(groups.len() as i64);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::command_executor::MockCommandExecutor;

    #[fixture]
    fn group() -> String {
        r#"
[
  {
    "id": "??????????????????????????",
    "name": "Recovery",
    "description": "Can reset user passwords if account recovery is enabled.",
    "state": "ACTIVE",
    "created_at": "2023-03-19T05:06:31Z"
  },
  {
    "id": "??????????????????????????",
    "name": "Owners",
    "description": "Access to billing and account administration.",
    "state": "ACTIVE",
    "created_at": "2023-03-19T05:06:31Z"
  },
  {
    "id": "??????????????????????????",
    "name": "Administrators",
    "description": "Administration of users, groups, and vaults.",
    "state": "ACTIVE",
    "created_at": "2023-03-19T05:06:31Z"
  },
  {
    "id": "??????????????????????????",
    "name": "Team Members",
    "description": "All team members.",
    "state": "ACTIVE",
    "created_at": "2023-03-19T05:06:31Z"
  }
]
"#
        .to_string()
    }

    #[rstest]
    fn test_read_group(group: String) {
        // Arrange
        let mut command_executor = MockCommandExecutor::new();
        command_executor
            .expect_exec()
            .with(eq(vec!["group", "list", "--format", "json"]))
            .returning(move |_| Ok(group.clone()));
        let collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        collector.read_group();

        // Assert
        assert_eq!(
            OP_GROUP_COUNT_TOTAL
                .get_metric_with_label_values(&[])
                .unwrap()
                .get(),
            4
        );
    }
}
