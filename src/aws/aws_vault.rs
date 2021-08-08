use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::convert::TryFrom;
use tokio_postgres::Row;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AwsVault {
    pub creation_date: DateTime<FixedOffset>,
    pub last_inventory_date: Option<DateTime<FixedOffset>>,
    pub number_of_archives: i64,
    pub size_in_bytes: i64,
    #[serde(rename(deserialize = "VaultARN"))]
    pub vault_arn: String,
    pub vault_name: String,
}

impl TryFrom<&str> for AwsVault {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(|e| e.into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AwsVaultListResponse {
    pub vault_list: Vec<AwsVault>,
    pub marker: Option<String>,
}

impl TryFrom<&Row> for AwsVault {
    type Error = anyhow::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(AwsVault {
            creation_date: value.try_get("creation_date")?,
            last_inventory_date: value.try_get("last_inventory_date")?,
            number_of_archives: value.try_get("number_of_archives")?,
            size_in_bytes: value.try_get("size_in_bytes")?,
            vault_arn: value.try_get("vault_arn")?,
            vault_name: value.try_get("vault_name")?,
        })
    }
}
