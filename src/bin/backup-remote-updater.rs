extern crate backup_remote_rs;
use anyhow::Result;
use backup_remote_rs::repo::Repository;
use backup_remote_rs::{aws::{aws_glacier::AwsGlacier, aws_vault::AwsVault}};
extern crate clap;
use clap::{App, Arg};
use log::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

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
    debug!("creating aws glacier object");
    let aws_glacier = AwsGlacier::new(
        matches.value_of("secret_key").unwrap(),
        matches.value_of("key_id").unwrap(),
        matches.value_of("region").unwrap(),
    );
    debug!("creating repository object");
    let mut repo = Repository::new(matches.value_of("db_connection").unwrap()).await?;
    let aws_vaults = aws_glacier.list_vaults().await?;
    debug!("found {} aws vaults", aws_vaults.len());
    let trans = repo.get_transaction().await?;
    let repo_vaults = Repository::get_vaults(&trans).await?;
    debug!("found {} repository vaults", repo_vaults.len());
    let mut vaults = Vec::<AwsVault>::new();

    for vault in aws_vaults {
        match repo_vaults.iter().find(|&v| v.vault_arn == vault.vault_arn) {
            None => {
                vaults.push(
                    Repository::create_vault(&trans, &vault)
                    .await?,
                );
                info!("added vault \"{}\" to repository", vault.vault_name);
            }
            Some(v) => {
                vaults.push(Repository::update_vault(&trans, v).await?);
                info!("updated vault \"{}\" in repository", v.vault_name);
            }
        }

        let job_list = aws_glacier.list_jobs_for_vault(&vault).await?;
        println!("{:?}", job_list);
    }

    trans.commit().await?;
    
    // If the inventory of a vault is older than 1 week, launch an inventory job

    // Check inventory jobs and launch workers as needed

    // Check download jobs and launch workers as needed

    // Check upload jobs and launch workers as needed

    Ok(())
}
