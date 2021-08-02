extern crate backup_remote_rs;
use anyhow::Result;
use backup_remote_rs::repo::Repository;
use backup_remote_rs::{aws::aws_glacier::AwsGlacier, repo::repo_vault::RepoVault};
extern crate clap;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<()> {
    // Process arguments
    let matches = App::new("backup-remote-updater")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("secret_key")
                .required(true)
                .env("AWS_SECRET_KEY"),
        )
        .arg(Arg::with_name("key_id").required(true).env("AWS_KEY_ID"))
        .arg(Arg::with_name("region").required(true).env("AWS_REGION"))
        .arg(
            Arg::with_name("db_connection")
                .required(true)
                .env("DB_CONNECTION"),
        )
        .get_matches();

    // Update list of vaults
    let aws_glacier = AwsGlacier::new(
        matches.value_of("secret_key").unwrap(),
        matches.value_of("key_id").unwrap(),
        matches.value_of("region").unwrap(),
    );
    let repo = Repository::new(matches.value_of("db_connection").unwrap()).await?;
    let aws_vaults = aws_glacier.list_vaults().await?;
    let repo_vaults = repo.get_vaults().await?;
    let mut vaults = Vec::<RepoVault>::new();

    for vault in aws_vaults {
        match repo_vaults.iter().find(|&v| v.vault_arn == vault.vault_arn) {
            None => {
                vaults.push(
                    repo.create_vault(
                        &vault.creation_date,
                        &vault.inventory_date,
                        &vault.number_of_archives,
                        &vault.size_in_bytes,
                        &vault.vault_arn,
                        &vault.vault_name,
                    )
                    .await?,
                );
            }
            Some(v) => {
                let mut v_new = v.clone();

                v_new.creation_date = vault.creation_date;
                v_new.inventory_date = vault.inventory_date;
                v_new.number_of_archives = vault.number_of_archives;
                v_new.size_in_bytes = vault.size_in_bytes;
                v_new.vault_name = vault.vault_name;

                vaults.push(repo.update_vault(&v_new).await?);
            }
        }
    }

    println!("{:?}", vaults);

    // If the inventory of a vault is older than 1 week, launch an inventory job

    // Check inventory jobs and launch workers as needed

    // Check download jobs and launch workers as needed

    // Check upload jobs and launch workers as needed

    Ok(())
}
