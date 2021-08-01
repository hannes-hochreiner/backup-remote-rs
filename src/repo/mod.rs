pub mod repo_vault;

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use std::{convert::TryFrom, str};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

use repo_vault::RepoVault;

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

    pub async fn get_vaults(&self) -> Result<Vec<RepoVault>> {
        let rows = self.client.query("SELECT id, revision, creation_date, inventory_date, number_of_archives, size_in_bytes, vault_arn, vault_name FROM vaults", &[]).await?;
        let mut res = Vec::<RepoVault>::new();

        for row in rows {
            res.push(RepoVault::try_from(&row)?);
        }

        Ok(res)
    }

    pub async fn create_vault(
        &self,
        creation_date: &DateTime<FixedOffset>,
        inventory_date: &Option<DateTime<FixedOffset>>,
        number_of_archives: &i64,
        size_in_bytes: &i64,
        vault_arn: &String,
        vault_name: &String,
    ) -> Result<RepoVault> {
        let rows = self.client.query(
            "INSERT INTO vaults (id, revision, creation_date, inventory_date, number_of_archives, size_in_bytes, vault_arn, vault_name) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *", 
            &[&Uuid::new_v4(), &Uuid::new_v4(), creation_date, inventory_date, number_of_archives, size_in_bytes, vault_arn, vault_name]
        ).await?;

        match rows.len() {
            1 => Ok(RepoVault::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating vault")),
        }
    }

    pub async fn update_vault(&self, vault: &RepoVault) -> Result<RepoVault> {
        let rows = self.client.query(
            "UPDATE vaults SET revision=$1, creation_date=$2, inventory_date=$3, number_of_archives=$4, size_in_bytes=$5, vault_arn=$6, vault_name=$7 WHERE id=$8 AND revision=$9 RETURNING *", 
            &[&Uuid::new_v4(), &vault.creation_date, &vault.inventory_date, &vault.number_of_archives, &vault.size_in_bytes, &vault.vault_arn, &vault.vault_name, &vault.id, &vault.revision]
        ).await?;

        match rows.len() {
            1 => Ok(RepoVault::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating vault")),
        }
    }
}
