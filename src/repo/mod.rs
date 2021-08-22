pub mod repo_vault;
pub mod repo_job;
pub mod repo_archive;

use anyhow::Result;
use std::str;
use tokio_postgres::{Client, NoTls, Transaction};

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
}
