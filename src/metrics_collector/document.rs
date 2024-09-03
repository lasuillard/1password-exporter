use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_DOCUMENT_COUNT_TOTAL: IntGaugeVec =
        register_int_gauge_vec!("op_document_count_total", "Total number of documents.", &[])
            .unwrap();
}

#[derive(Deserialize, Debug)]
struct Document {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) title: String,
    #[allow(dead_code)]
    pub(crate) version: i32,
    #[allow(dead_code)]
    pub(crate) vault: DocumentVault,
    #[allow(dead_code)]
    pub(crate) last_edited_by: String,
    #[allow(dead_code)]
    pub(crate) created_at: String,
    #[allow(dead_code)]
    pub(crate) updated_at: String,
}

#[derive(Deserialize, Debug)]
struct DocumentVault {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) name: String,
}

impl OpMetricsCollector {
    pub(crate) fn read_document(&self) {
        let output = self
            .command_executor
            .exec(vec![
                "document",
                "list",
                "--format",
                "json",
                "--include-archive",
            ])
            .unwrap();
        let documents: Vec<Document> = serde_json::from_str(&output).unwrap();

        OP_DOCUMENT_COUNT_TOTAL
            .with_label_values(&[])
            .set(documents.len() as i64);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_document(metrics_collector: OpMetricsCollector) {
        // Act
        metrics_collector.read_document();

        // Assert
        assert_eq!(
            OP_DOCUMENT_COUNT_TOTAL
                .get_metric_with_label_values(&[])
                .unwrap()
                .get(),
            1
        );
    }
}
