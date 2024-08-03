use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_EXPORTER_BUILDINFO: IntGaugeVec = register_int_gauge_vec!(
        "op_exporter_buildinfo",
        "1Password CLI build information.",
        &["version"]
    )
    .unwrap();
}

impl OpMetricsCollector {
    pub(crate) fn read_buildinfo(&self) {
        OP_EXPORTER_BUILDINFO
            .with_label_values(&[env!("CARGO_PKG_VERSION")])
            .set(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_executor::MockCommandExecutor;

    #[test]
    fn test_read_buildinfo() {
        // Arrange
        let command_executor = MockCommandExecutor::new();
        let collector = OpMetricsCollector::new(Box::new(command_executor));

        // Act
        collector.read_buildinfo();

        // Assert
        assert_eq!(
            OP_EXPORTER_BUILDINFO
                .get_metric_with_label_values(&["0.1.0"])
                .unwrap()
                .get(),
            1
        );
    }
}
