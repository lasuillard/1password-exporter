use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_EXPORTER_BUILDINFO: IntGaugeVec = register_int_gauge_vec!(
        "op_exporter_buildinfo",
        "Build information of this exporter.",
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
    use anyhow::Result;
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_buildinfo(metrics_collector: OpMetricsCollector) -> Result<()> {
        // Act
        metrics_collector.read_buildinfo();

        // Assert
        assert_eq!(
            OP_EXPORTER_BUILDINFO
                .get_metric_with_label_values(&["0.4.2"])?
                .get(),
            1
        );

        Ok(())
    }
}
