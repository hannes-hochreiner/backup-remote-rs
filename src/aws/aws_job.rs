use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize};
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
    #[serde(rename(serialize = "VaultARN"))]
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
