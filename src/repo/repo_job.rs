use super::Repository;
use crate::aws::aws_job::AwsJob;
use anyhow::Result;
use log::debug;
use std::convert::TryFrom;
use tokio_postgres::Transaction;

impl Repository {
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
