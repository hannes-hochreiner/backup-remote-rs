use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::convert::TryFrom;
use tokio_postgres::Row;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AwsJob {
    pub job_id: String,
    pub action: String,
    pub archive_id: Option<String>,
    pub archive_tree_hash: Option<String>,
    pub archive_size_in_bytes: Option<i64>,
    pub completion_date: Option<DateTime<FixedOffset>>,
    pub creation_date: DateTime<FixedOffset>,
    pub inventory_size_in_bytes: Option<i64>,
    pub job_description: Option<String>,
    pub tree_hash: Option<String>,
    pub status_code: String,
    pub status_message: Option<String>,
    #[serde(rename(deserialize = "VaultARN"))]
    pub vault_arn: String,
}

impl TryFrom<&str> for AwsJob {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(|e| e.into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AwsJobListResponse {
    pub job_list: Vec<AwsJob>,
    pub marker: Option<String>,
}

impl TryFrom<&Row> for AwsJob {
    type Error = anyhow::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(AwsJob {
            job_id: value.try_get("job_id")?,
            action: value.try_get("action")?,
            archive_id: value.try_get("archive_id")?,
            archive_tree_hash: value.try_get("archive_tree_hash")?,
            archive_size_in_bytes: value.try_get("archive_size_in_bytes")?,
            completion_date: value.try_get("completion_date")?,
            creation_date: value.try_get("creation_date")?,
            inventory_size_in_bytes: value.try_get("inventory_size_in_bytes")?,
            job_description: value.try_get("job_description")?,
            tree_hash: value.try_get("tree_hash")?,
            status_code: value.try_get("status_code")?,
            status_message: value.try_get("status_message")?,
            vault_arn: value.try_get("vault_arn")?,
        })
    }
}
