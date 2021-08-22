use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::convert::TryFrom;
use tokio_postgres::Row;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AwsArchive {
    pub archive_id: String,
    pub archive_description: String,
    pub creation_date: DateTime<FixedOffset>,
    pub size: i64,
    #[serde(rename(deserialize = "SHA256TreeHash"))]
    pub tree_hash: String,
}

impl TryFrom<&str> for AwsArchive {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(|e| e.into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AwsIventoryResponse {
    #[serde(rename(deserialize = "VaultARN"))]
    pub vault_arn: String,
    pub inventory_date: DateTime<FixedOffset>,
    pub archive_list: Vec<AwsArchive>,
}

impl TryFrom<&Row> for AwsArchive {
    type Error = anyhow::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(AwsArchive {
            archive_id: value.try_get("archive_id")?,
            archive_description: value.try_get("archive_description")?,
            creation_date: value.try_get("creation_date")?,
            size: value.try_get("size")?,
            tree_hash: value.try_get("tree_hash")?,
        })
    }
}
