use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use std::convert::TryFrom;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug)]
pub struct RepoVault {
    id: Uuid,
    revision: Uuid,
    creation_date: DateTime<FixedOffset>,
    inventory_date: Option<DateTime<FixedOffset>>,
    number_of_archives: i64,
    size_in_bytes: i64,
    vault_arn: String,
    vault_name: String,
}

impl TryFrom<&Row> for RepoVault {
    type Error = anyhow::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(RepoVault {
            id: value.try_get("id")?,
            revision: value.try_get("revision")?,
            creation_date: value.try_get("creation_date")?,
            inventory_date: value.try_get("inventory_date")?,
            number_of_archives: value.try_get("number_of_archives")?,
            size_in_bytes: value.try_get("size_in_bytes")?,
            vault_arn: value.try_get("vault_arn")?,
            vault_name: value.try_get("vault_name")?,
        })
    }
}
