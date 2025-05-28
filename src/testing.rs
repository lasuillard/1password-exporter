#![cfg(test)]

use mockall::predicate::*;
use rstest::fixture;

use crate::{command_executor::MockCommandExecutor, metrics_collector::OpMetricsCollector, test_dir};

/// Returns a mock command executor with mock responses set.
#[fixture]
pub(crate) fn command_executor() -> MockCommandExecutor {
    let mut command_executor = MockCommandExecutor::new();

    let mock_commands = vec![
        (
            vec!["account", "get", "--format", "json"],
            include_str!(test_dir!("fixtures/account.json")),
        ),
        (
            vec!["document", "list", "--format", "json", "--include-archive"],
            include_str!(test_dir!("fixtures/document.json")),
        ),
        (
            vec!["group", "list", "--format", "json"],
            include_str!(test_dir!("fixtures/group.json")),
        ),
        (
            vec!["item", "list", "--format", "json", "--include-archive"],
            include_str!(test_dir!("fixtures/item.json")),
        ),
        (
            vec!["service-account", "ratelimit", "--format", "json"],
            include_str!(test_dir!("fixtures/ratelimit.json")),
        ),
        (
            vec!["whoami", "--format", "json"],
            include_str!(test_dir!("fixtures/whoami.json")),
        ),
        (
            vec!["user", "list", "--format", "json"],
            include_str!(test_dir!("fixtures/user.json")),
        ),
        (
            vec!["vault", "list", "--format", "json"],
            include_str!(test_dir!("fixtures/vault.json")),
        ),
    ];

    for (args, output) in mock_commands {
        command_executor
            .expect_exec()
            .with(eq(args))
            .returning(move |_| Ok(output.to_string()));
    }

    command_executor
}

#[fixture]
pub(crate) fn metrics_collector(command_executor: MockCommandExecutor) -> OpMetricsCollector {
    OpMetricsCollector::new(Box::new(command_executor))
}
