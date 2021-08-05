pub mod repo_vault;

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use std::{convert::TryFrom, str};
use tokio_postgres::{Client, NoTls, Transaction};
use uuid::Uuid;
use log::{debug};

use crate::aws::aws_vault::AwsVault;

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

    pub async fn create_vault(
        transaction: &Transaction<'_>,
        vault: &AwsVault,
    ) -> Result<AwsVault> {
        debug!("creating new vault");
        let rows = transaction.query(
            "INSERT INTO vaults (creation_date, inventory_date, number_of_archives, size_in_bytes, vault_arn, vault_name) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *", 
            &[&vault.creation_date, &vault.inventory_date, &vault.number_of_archives, &vault.size_in_bytes, &vault.vault_arn, &vault.vault_name]
        ).await?;

        match rows.len() {
            1 => Ok(AwsVault::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating vault")),
        }
    }

    pub async fn update_vault(transaction: &Transaction<'_>, vault: &AwsVault) -> Result<AwsVault> {
        debug!("updating vault");
        let rows = transaction.query(
            "UPDATE vaults SET creation_date=$1, inventory_date=$2, number_of_archives=$3, size_in_bytes=$4, vault_name=$5 WHERE vault_arn=$6 RETURNING *", 
            &[&vault.creation_date, &vault.inventory_date, &vault.number_of_archives, &vault.size_in_bytes, &vault.vault_name, &vault.vault_arn]
        ).await?;

        match rows.len() {
            1 => Ok(AwsVault::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating vault")),
        }
    }
}
