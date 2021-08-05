use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;
use std::convert::TryFrom;
use tokio_postgres::Row;

#[derive(Debug)]
pub struct AwsVault {
    pub creation_date: DateTime<FixedOffset>,
    pub inventory_date: Option<DateTime<FixedOffset>>,
    pub number_of_archives: i64,
    pub size_in_bytes: i64,
    pub vault_arn: String,
    pub vault_name: String,
}

impl TryFrom<&Value> for AwsVault {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(AwsVault {
            creation_date: DateTime::parse_from_rfc3339(
                value["CreationDate"]
                    .as_str()
                    .ok_or(anyhow::Error::msg("creation date not found"))?,
            )
            .context("error parsing creation date")?,
            inventory_date: match value["LastInventoryDate"].as_str() {
                Some(date_str) => Some(
                    DateTime::parse_from_rfc3339(date_str)
                        .context("error parsing creation date")?,
                ),
                None => None,
            },
            number_of_archives: value["NumberOfArchives"]
                .as_i64()
                .ok_or(anyhow::Error::msg("number of archives not found"))?,
            size_in_bytes: value["SizeInBytes"]
                .as_i64()
                .ok_or(anyhow::Error::msg("size in bytes not found"))?,
            vault_arn: value["VaultARN"]
                .as_str()
                .ok_or(anyhow::Error::msg("vault arn not found"))?
                .into(),
            vault_name: value["VaultName"]
                .as_str()
                .ok_or(anyhow::Error::msg("vault name not found"))?
                .into(),
        })
    }
}

impl TryFrom<&Row> for AwsVault {
    type Error = anyhow::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(AwsVault {
            creation_date: value.try_get("creation_date")?,
            inventory_date: value.try_get("inventory_date")?,
            number_of_archives: value.try_get("number_of_archives")?,
            size_in_bytes: value.try_get("size_in_bytes")?,
            vault_arn: value.try_get("vault_arn")?,
            vault_name: value.try_get("vault_name")?,
        })
    }
}
