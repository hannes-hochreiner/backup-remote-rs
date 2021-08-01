use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use std::convert::TryFrom;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RepoVault {
    pub id: Uuid,
    pub revision: Uuid,
    pub creation_date: DateTime<FixedOffset>,
    pub inventory_date: Option<DateTime<FixedOffset>>,
    pub number_of_archives: i64,
    pub size_in_bytes: i64,
    pub vault_arn: String,
    pub vault_name: String,
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
