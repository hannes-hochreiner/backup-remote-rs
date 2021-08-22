pub mod repo_archive;

use anyhow::Result;
use log::debug;
use std::{convert::TryFrom, str};
use tokio_postgres::{Client, NoTls, Transaction};

use crate::aws::{aws_job::AwsJob, aws_vault::AwsVault};

pub struct Repository {
    client: Client,
}

impl Repository {
    pub async fn new(config: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(config, NoTls).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Repository { client })
    }

    pub async fn get_transaction(&mut self) -> Result<Transaction<'_>> {
        self.client.transaction().await.map_err(|e| e.into())
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

    pub async fn create_job(transaction: &Transaction<'_>, job: &AwsJob) -> Result<AwsJob> {
        debug!("creating new job");
        let rows = transaction.query(
            "INSERT INTO jobs (job_id, action, archive_id, archive_tree_hash, archive_size_in_bytes, completion_date, creation_date, inventory_size_in_bytes, job_description, tree_hash, status_code, status_message, vault_arn) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING *", 
            &[&job.job_id, &job.action, &job.archive_id, &job.archive_tree_hash, &job.archive_size_in_bytes, &job.completion_date, &job.creation_date, &job.inventory_size_in_bytes, &job.job_description, &job.tree_hash, &job.status_code, &job.status_message, &job.vault_arn]
        ).await?;

        match rows.len() {
            1 => Ok(AwsJob::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating job")),
        }
    }

    pub async fn update_job(transaction: &Transaction<'_>, job: &AwsJob) -> Result<AwsJob> {
        debug!("updating job");
        let rows = transaction.query(
            "UPDATE jobs SET action=$2, archive_id=$3, archive_tree_hash=$4, archive_size_in_bytes=$5, completion_date=$6, creation_date=$7, inventory_size_in_bytes=$8, job_description=$9, tree_hash=$10, status_code=$11, status_message=$12, vault_arn=$13 WHERE job_id=$1 RETURNING *", 
            &[&job.job_id, &job.action, &job.archive_id, &job.archive_tree_hash, &job.archive_size_in_bytes, &job.completion_date, &job.creation_date, &job.inventory_size_in_bytes, &job.job_description, &job.tree_hash, &job.status_code, &job.status_message, &job.vault_arn]
        ).await?;

        match rows.len() {
            1 => Ok(AwsJob::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating job")),
        }
    }

    pub async fn get_job_by_id(transaction: &Transaction<'_>, job_id: &str) -> Result<AwsJob> {
        debug!("getting job \"{}\"", job_id);
        let rows = transaction
            .query("SELECT * from jobs WHERE job_id=$1", &[&job_id])
            .await?;

        match rows.len() {
            1 => Ok(AwsJob::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error getting job by id")),
        }
    }

    pub async fn get_latest_job_by_action_vault(
        transaction: &Transaction<'_>,
        action: &str,
        vault_arn: &str,
    ) -> Result<AwsJob> {
        debug!(
            "getting job by action \"{}\" and vault \"{}\"",
            action, vault_arn
        );
        let rows = transaction.query(
            "SELECT * from jobs WHERE action=$1 AND vault_arn=$2 ORDER BY creation_date DESC LIMIT 1", 
            &[&action, &vault_arn]
        ).await?;

        match rows.len() {
            1 => Ok(AwsJob::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg(
                "error getting latest job by action and vault",
            )),
        }
    }

    pub async fn reset_jobs_status_active(transaction: &Transaction<'_>) -> Result<()> {
        transaction
            .query("UPDATE jobs_status SET active=FALSE", &[])
            .await?;
        Ok(())
    }

    pub async fn set_job_status_active(transaction: &Transaction<'_>, job: &AwsJob) -> Result<()> {
        let rows = transaction
            .query("SELECT * FROM jobs_status WHERE job_id=$1", &[&job.job_id])
            .await?;

        match rows.len() {
            0 => {
                transaction
                    .query(
                        "INSERT INTO jobs_status (job_id, active) VALUES ($1, TRUE)",
                        &[&job.job_id],
                    )
                    .await?;
                Ok(())
            }
            1 => {
                transaction
                    .query(
                        "UPDATE jobs_status SET active=TRUE WHERE job_id=$1",
                        &[&job.job_id],
                    )
                    .await?;
                Ok(())
            }
            _ => Err(anyhow::Error::msg("error updating job status active")),
        }
    }
}
