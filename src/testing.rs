#![cfg(test)]

use mockall::predicate::*;
use rstest::fixture;

use crate::{command_executor::MockCommandExecutor, metrics_collector::OpMetricsCollector};

const ACCOUNT_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/account.json"
));

const DOCUMENT_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/document.json"
));

/// Returns a mock command executor with mock responses set.
#[fixture]
pub(crate) fn command_executor() -> MockCommandExecutor {
    let mut command_executor = MockCommandExecutor::new();

    let mock_commands = vec![
        (vec!["account", "get", "--format", "json"], ACCOUNT_FIXTURE),
        (
            vec!["document", "list", "--format", "json", "--include-archive"],
            DOCUMENT_FIXTURE,
        ),
        (
            vec!["group", "list", "--format", "json"],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/group.json"
            )),
        ),
        (
            vec!["item", "list", "--format", "json", "--include-archive"],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/item.json"
            )),
        ),
        (
            vec!["service-account", "ratelimit", "--format", "json"],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ratelimit.json"
            )),
        ),
        (
            vec!["whoami", "--format", "json"],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/whoami.json"
            )),
        ),
        (
            vec!["user", "list", "--format", "json"],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/user.json"
            )),
        ),
        (
            vec!["vault", "list", "--format", "json"],
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/vault.json"
            )),
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
