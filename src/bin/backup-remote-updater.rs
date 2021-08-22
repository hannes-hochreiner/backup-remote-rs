extern crate backup_remote_rs;
use anyhow::Result;
use backup_remote_rs::aws::{aws_glacier::AwsGlacier, aws_vault::AwsVault};
use backup_remote_rs::repo::Repository;
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
    let trans = repo.get_transaction().await?;
    // Reset vault status active
    debug!("resetting vault status active");
    Repository::reset_vaults_status_active(&trans).await?;

    let aws_vaults = aws_glacier.list_vaults().await?;
    debug!("found {} aws vaults", aws_vaults.len());
    let repo_vaults = Repository::get_vaults(&trans).await?;
    debug!("found {} repository vaults", repo_vaults.len());
    let mut vaults = Vec::<AwsVault>::new();

    for vault in aws_vaults {
        match repo_vaults.iter().find(|&v| v.vault_arn == vault.vault_arn) {
            None => {
                vaults.push(Repository::create_vault(&trans, &vault).await?);
                info!("added vault \"{}\" to repository", vault.vault_name);
            }
            Some(v) => {
                vaults.push(Repository::update_vault(&trans, v).await?);
                info!("updated vault \"{}\" in repository", v.vault_name);
            }
        }

        // set vault status active
        Repository::set_vault_status_active(&trans, &vault).await?;

        // update the list of jobs for this vault
        Repository::reset_jobs_status_active(&trans).await?;
        let aws_jobs = aws_glacier.list_jobs_for_vault(&vault).await?;

        for job in aws_jobs {
            Repository::set_job_status_active(&trans, &job).await?;

            match Repository::get_job_by_id(&trans, &*job.job_id).await {
                Ok(_) => Repository::update_job(&trans, &job).await?,
                Err(_) => Repository::create_job(&trans, &job).await?,
            };
        }

        // get the latest inventory job for this vault
        // if the job is older then the inventory date of the vault => launch new inventory job
        match vault.last_inventory_date {
            Some(inv_date) => {
                debug!("inventory date found for vault \"{}\"", vault.vault_name);
                if match Repository::get_latest_job_by_action_vault(
                    &trans,
                    "InventoryRetrieval",
                    &*vault.vault_arn,
                )
                .await
                {
                    Ok(latest_job) => {
                        if latest_job.creation_date < inv_date {
                            debug!("latest inventory job date older than inventory date of vault");
                            true
                        } else {
                            false
                        }
                    }
                    Err(_) => true,
                } {
                    // launch inventory job
                    debug!("creating inventory job for \"{}\"", vault.vault_name);
                    let job_id = aws_glacier.init_inventory_job_for_vault(&vault).await?;
                    info!(
                        "created inventory job for \"{}\" with id \"{}\"",
                        vault.vault_name, job_id
                    );

                    // add job to repository
                    let job = aws_glacier.get_job_by_id_vault(&vault, &*job_id).await?;
                    Repository::create_job(&trans, &job).await?;
                    Repository::set_job_status_active(&trans, &job).await?;
                }
            }
            None => {
                debug!("no inventory date found for vault \"{}\"", vault.vault_name);
            }
        }
    }

    trans.commit().await?;

    Ok(())
}
