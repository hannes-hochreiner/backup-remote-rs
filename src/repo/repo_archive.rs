use super::Repository;
use crate::aws::aws_archive::AwsArchive;
use anyhow::Result;
use log::debug;
use std::convert::TryFrom;
use tokio_postgres::Transaction;

impl Repository {
    pub async fn create_archive(transaction: &Transaction<'_>, archive: &AwsArchive) -> Result<AwsArchive> {
        debug!("creating new archive");
        let rows = transaction.query(
            "INSERT INTO archives (archive_id, archive_description, creation_date, size, tree_hash) VALUES ($1, $2, $3, $4, $5) RETURNING *", 
            &[&archive.archive_id, &archive.archive_description, &archive.creation_date, &archive.size, &archive.tree_hash]
        ).await?;

        match rows.len() {
            1 => Ok(AwsArchive::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating archive")),
        }
    }

    pub async fn update_archive(transaction: &Transaction<'_>, archive: &AwsArchive) -> Result<AwsArchive> {
        debug!("updating archive");
        let rows = transaction.query(
            "UPDATE archives SET archive_description=$2, creation_date=$3, size=$4, tree_hash=$5 WHERE archive_id=$1 RETURNING *", 
            &[&archive.archive_id, &archive.archive_description, &archive.creation_date, &archive.size, &archive.tree_hash]
        ).await?;

        match rows.len() {
            1 => Ok(AwsArchive::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating archive")),
        }
    }
}
