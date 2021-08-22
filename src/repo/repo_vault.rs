use super::Repository;
use crate::aws::{aws_archive::AwsArchive, aws_vault::AwsVault};
use anyhow::Result;
use log::debug;
use std::convert::TryFrom;
use tokio_postgres::Transaction;

impl Repository {
    pub async fn create_vault(transaction: &Transaction<'_>, vault: &AwsVault) -> Result<AwsVault> {
        debug!("creating new vault");
        let rows = transaction.query(
            "INSERT INTO vaults (creation_date, last_inventory_date, number_of_archives, size_in_bytes, vault_arn, vault_name) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *", 
            &[&vault.creation_date, &vault.last_inventory_date, &vault.number_of_archives, &vault.size_in_bytes, &vault.vault_arn, &vault.vault_name]
        ).await?;

        match rows.len() {
            1 => Ok(AwsVault::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating vault")),
        }
    }

    pub async fn update_vault(transaction: &Transaction<'_>, vault: &AwsVault) -> Result<AwsVault> {
        debug!("updating vault");
        let rows = transaction.query(
            "UPDATE vaults SET creation_date=$1, last_inventory_date=$2, number_of_archives=$3, size_in_bytes=$4, vault_name=$5 WHERE vault_arn=$6 RETURNING *", 
            &[&vault.creation_date, &vault.last_inventory_date, &vault.number_of_archives, &vault.size_in_bytes, &vault.vault_name, &vault.vault_arn]
        ).await?;

        match rows.len() {
            1 => Ok(AwsVault::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating vault")),
        }
    }

    pub async fn get_vaults(transaction: &Transaction<'_>) -> Result<Vec<AwsVault>> {
        debug!("getting vaults");
        let rows = transaction.query("SELECT * FROM vaults", &[]).await?;
        let mut res = Vec::<AwsVault>::new();

        for row in rows {
            res.push(AwsVault::try_from(&row)?);
        }

        Ok(res)
    }

    pub async fn reset_vaults_status_active(transaction: &Transaction<'_>) -> Result<()> {
        transaction
            .query("UPDATE vaults_status SET active=FALSE", &[])
            .await?;
        Ok(())
    }

    pub async fn set_vault_status_active(
        transaction: &Transaction<'_>,
        vault: &AwsVault,
    ) -> Result<()> {
        let rows = transaction
            .query(
                "SELECT * FROM vaults_status WHERE vault_arn=$1",
                &[&vault.vault_arn],
            )
            .await?;

        match rows.len() {
            0 => {
                transaction
                    .query(
                        "INSERT INTO vaults_status (vault_arn, active) VALUES ($1, TRUE)",
                        &[&vault.vault_arn],
                    )
                    .await?;
                Ok(())
            }
            1 => {
                transaction
                    .query(
                        "UPDATE vaults_status SET active=TRUE WHERE vault_arn=$1",
                        &[&vault.vault_arn],
                    )
                    .await?;
                Ok(())
            }
            _ => Err(anyhow::Error::msg("error updating vault status active")),
        }
    }

    pub async fn delete_archive_associations(
        transaction: &Transaction<'_>,
        vault: &AwsVault,
    ) -> Result<()> {
        debug!(
            "deleting archive associations for vault \"{}\"",
            &vault.vault_name
        );
        transaction
            .query(
                "DELETE FROM vaults_archives WHERE vault_arn=$1",
                &[&vault.vault_arn],
            )
            .await?;
        Ok(())
    }

    pub async fn create_archive_association(
        transaction: &Transaction<'_>,
        vault: &AwsVault,
        archive: &AwsArchive,
    ) -> Result<()> {
        debug!(
            "creating associations to archive \"{}\" for vault \"{}\"",
            &archive.archive_id, &vault.vault_name
        );
        transaction
            .query(
                "INSERT INTO vaults_archives (vault_arn, archive_id) VALUES ($1, $2)",
                &[&vault.vault_arn, &archive.archive_id],
            )
            .await?;
        Ok(())
    }
}
