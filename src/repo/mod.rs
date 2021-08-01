pub mod repo_vault;

use anyhow::Result;
use std::{convert::TryFrom, str};
use tokio_postgres::{Client, NoTls};

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
}
