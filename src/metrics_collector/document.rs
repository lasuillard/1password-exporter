use std::collections::HashMap;

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
    static ref OP_DOCUMENT_COUNT_PER_VAULT: IntGaugeVec = register_int_gauge_vec!(
        "op_document_count_per_vault",
        "Number of documents per vault.",
        &["vault"]
    )
    .unwrap();
    static ref OP_DOCUMENT_COUNT_PER_TAG: IntGaugeVec = register_int_gauge_vec!(
        "op_document_count_per_tag",
        "Number of documents per tag.",
        &["tag"]
    )
    .unwrap();
    static ref OP_DOCUMENT_FILE_SIZE_PER_VAULT: IntGaugeVec = register_int_gauge_vec!(
        "op_document_file_size_per_vault_bytes",
        "Size of file in documents per vault, in bytes.",
        &["vault"],
    )
    .unwrap();
    static ref OP_DOCUMENT_FILE_SIZE_PER_TAG: IntGaugeVec = register_int_gauge_vec!(
        "op_document_file_size_per_tag_bytes",
        "Size of file in documents per tag, in bytes.",
        &["tag"],
    )
    .unwrap();
}

#[derive(Deserialize, Debug)]
struct Document {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) title: String,
    pub(crate) tags: Option<Vec<String>>,
    #[allow(dead_code)]
    pub(crate) version: i32,
    pub(crate) vault: DocumentVault,
    #[serde(rename = "overview.ainfo")]
    pub(crate) overview_ainfo: Option<String>,
    #[allow(dead_code)]
    pub(crate) last_edited_by: String,
    #[allow(dead_code)]
    pub(crate) created_at: String,
    #[allow(dead_code)]
    pub(crate) updated_at: String,
}

#[derive(Deserialize, Debug)]
struct DocumentVault {
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

        // Gather metrics
        let mut count_per_vault = HashMap::new();
        let mut count_per_tag = HashMap::new();
        let mut file_size_per_vault = HashMap::new();
        let mut file_size_per_tag = HashMap::new();

        for document in &documents {
            let vault_id = document.vault.id.clone();
            let file_size = match &document.overview_ainfo {
                Some(overview_ainfo) => match parse_file_size_bytes(overview_ainfo) {
                    Some(file_size) => file_size,
                    None => 0,
                },
                None => 0,
            };
            let tags = document.tags.clone().unwrap_or_default();

            *count_per_vault.entry(vault_id.clone()).or_insert(0) += 1;
            *file_size_per_vault.entry(vault_id).or_insert(0) += file_size;

            for tag in tags {
                *count_per_tag.entry(tag.clone()).or_insert(0) += 1;
                *file_size_per_tag.entry(tag).or_insert(0) += file_size;
            }
        }

        // Set metrics
        OP_DOCUMENT_COUNT_TOTAL
            .with_label_values(&[])
            .set(documents.len() as i64);

        count_per_vault.iter().for_each(|(vault, count)| {
            OP_DOCUMENT_COUNT_PER_VAULT
                .with_label_values(&[vault])
                .set(*count);
        });
        count_per_tag.iter().for_each(|(tag, count)| {
            OP_DOCUMENT_COUNT_PER_TAG
                .with_label_values(&[tag])
                .set(*count);
        });
        file_size_per_vault.iter().for_each(|(vault, size)| {
            OP_DOCUMENT_FILE_SIZE_PER_VAULT
                .with_label_values(&[vault])
                .set(*size);
        });
        file_size_per_tag.iter().for_each(|(tag, size)| {
            OP_DOCUMENT_FILE_SIZE_PER_TAG
                .with_label_values(&[tag])
                .set(*size);
        });
    }
}

/// Parse file size (e.g. `"10 bytes"`, `"1 KB"`) to bytes.
// * NOTE: Accurate file sizes not available in the output because 1Password truncate it when goes over KB.
fn parse_file_size_bytes(s: &String) -> Option<i64> {
    // Split size and unit
    let (size, unit) = match s.split_once(' ') {
        Some((size, unit)) => (size, unit),
        None => {
            log::error!("Invalid file size format: {}", s);
            return None;
        }
    };

    // Parse size
    let size = match size.parse::<i64>() {
        Ok(size) => size,
        Err(e) => {
            log::error!("Failed to parse file size: {}", e);
            return None;
        }
    };

    // Convert unit to multiplier
    let multiplier = match unit {
        "bytes" => 1,
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        _ => {
            log::error!("Unknown unit: {}", unit);
            return None;
        }
    };

    Some(size * multiplier)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_document(metrics_collector: OpMetricsCollector) -> Result<()> {
        // Act
        metrics_collector.read_document();

        // Assert
        assert_eq!(
            OP_DOCUMENT_COUNT_TOTAL
                .get_metric_with_label_values(&[])?
                .get(),
            4
        );
        assert_eq!(
            OP_DOCUMENT_COUNT_PER_VAULT
                .get_metric_with_label_values(&["36vhq4xz3r6hnemzadk33evi4a"])?
                .get(),
            4
        );
        assert_eq!(
            OP_DOCUMENT_COUNT_PER_TAG
                .get_metric_with_label_values(&["test"])?
                .get(),
            4
        );
        assert_eq!(
            OP_DOCUMENT_FILE_SIZE_PER_VAULT
                .get_metric_with_label_values(&["36vhq4xz3r6hnemzadk33evi4a"])?
                .get(),
            10494986
        );
        assert_eq!(
            OP_DOCUMENT_FILE_SIZE_PER_TAG
                .get_metric_with_label_values(&["test"])?
                .get(),
            10494986
        );

        Ok(())
    }
}
