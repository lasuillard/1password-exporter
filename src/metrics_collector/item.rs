use std::collections::HashMap;

use lazy_static::lazy_static;
#[cfg(test)]
use mockall::predicate::*;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use serde::Deserialize;

use super::OpMetricsCollector;

lazy_static! {
    static ref OP_ITEM_COUNT_TOTAL: IntGaugeVec =
        register_int_gauge_vec!("op_item_count_total", "Total number of items.", &[]).unwrap();
    static ref OP_ITEM_COUNT_PER_VAULT: IntGaugeVec = register_int_gauge_vec!(
        "op_item_count_per_vault",
        "Number of items per vault.",
        &["vault"]
    )
    .unwrap();
    static ref OP_ITEM_COUNT_PER_TAG: IntGaugeVec = register_int_gauge_vec!(
        "op_item_count_per_tag",
        "Number of items per tag.",
        &["tag"]
    )
    .unwrap();
    static ref OP_ITEM_COUNT_PER_CATEGORY: IntGaugeVec = register_int_gauge_vec!(
        "op_item_count_per_category",
        "Number of items per category.",
        &["category"]
    )
    .unwrap();
}

#[derive(Deserialize, Debug)]
struct Item {
    #[allow(dead_code)]
    pub(crate) id: String,
    #[allow(dead_code)]
    pub(crate) title: String,
    pub(crate) tags: Option<Vec<String>>,
    #[allow(dead_code)]
    pub(crate) version: i32,
    pub(crate) vault: ItemVault,
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

        // Gather metrics
        let mut count_per_vault = HashMap::new();
        let mut count_per_tag = HashMap::new();
        let mut count_per_category = HashMap::new();
        for item in &items {
            let vault_id = item.vault.id.clone();
            *count_per_vault.entry(vault_id).or_insert(0) += 1;

            let tags = item.tags.clone().unwrap_or_default();
            for tag in tags {
                *count_per_tag.entry(tag).or_insert(0) += 1;
            }

            let category = item.category.clone();
            *count_per_category.entry(category).or_insert(0) += 1;
        }

        // Set metrics
        OP_ITEM_COUNT_TOTAL
            .with_label_values(&[])
            .set(items.len() as i64);

        count_per_vault.iter().for_each(|(vault, count)| {
            OP_ITEM_COUNT_PER_VAULT
                .with_label_values(&[vault])
                .set(*count);
        });
        count_per_tag.iter().for_each(|(tag, count)| {
            OP_ITEM_COUNT_PER_TAG.with_label_values(&[tag]).set(*count);
        });
        count_per_category.iter().for_each(|(category, count)| {
            OP_ITEM_COUNT_PER_CATEGORY
                .with_label_values(&[category])
                .set(*count);
        });
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::*;

    use super::*;
    use crate::testing::metrics_collector;

    #[rstest]
    fn test_read_item(metrics_collector: OpMetricsCollector) -> Result<()> {
        metrics_collector.read_item();

        // Assert
        assert_eq!(
            OP_ITEM_COUNT_TOTAL.get_metric_with_label_values(&[])?.get(),
            5
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_VAULT
                .get_metric_with_label_values(&["36vhq4xz3r6hnemzadk33evi4a"])?
                .get(),
            5
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_TAG
                .get_metric_with_label_values(&["dev"])?
                .get(),
            1
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_TAG
                .get_metric_with_label_values(&["test"])?
                .get(),
            4
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_CATEGORY
                .get_metric_with_label_values(&["DOCUMENT"])?
                .get(),
            1
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_CATEGORY
                .get_metric_with_label_values(&["LOGIN"])?
                .get(),
            2
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_CATEGORY
                .get_metric_with_label_values(&["SECURE_NOTE"])?
                .get(),
            1
        );
        assert_eq!(
            OP_ITEM_COUNT_PER_CATEGORY
                .get_metric_with_label_values(&["SSH_KEY"])?
                .get(),
            1
        );

        Ok(())
    }
}
