use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct AwsVault {
    creation_date: DateTime<FixedOffset>,
    inventory_date: Option<DateTime<FixedOffset>>,
    number_of_archives: u64,
    size_in_bytes: u64,
    vault_arn: String,
    vault_name: String,
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
                .as_u64()
                .ok_or(anyhow::Error::msg("number of archives not found"))?,
            size_in_bytes: value["SizeInBytes"]
                .as_u64()
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
