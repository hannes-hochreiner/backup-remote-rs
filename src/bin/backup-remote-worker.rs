extern crate backup_remote_rs;
use anyhow::Result;
use backup_remote_rs::aws::aws_glacier::AwsGlacier;
use backup_remote_rs::repo::Repository;
extern crate clap;
use clap::{App, Arg};
use log::{debug, error, info};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Process arguments
    let matches = App::new("backup-remote-worker")
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

    let secret_key = matches.value_of("secret_key").unwrap();
    let key_id = matches.value_of("key_id").unwrap();
    let region = matches.value_of("region").unwrap();
    let db_connection = matches.value_of("db_connection").unwrap();

    loop {
        match update(secret_key, key_id, region, db_connection).await {
            Ok(_) => info!("update succeeded"),
            Err(e) => error!("{:?}", e),
        }

        sleep(Duration::from_secs(60 * 30)).await;
    }
}

async fn update(secret_key: &str, key_id: &str, region: &str, db_connection: &str) -> Result<()> {
    debug!("creating aws glacier object");
    let aws_glacier = AwsGlacier::new(secret_key, key_id, region);
    debug!("creating repository object");
    let mut repo = Repository::new(db_connection).await?;
    let aws_vaults = aws_glacier.list_vaults().await?;

    for vault in aws_vaults {
        let aws_jobs = aws_glacier.list_jobs_for_vault(&vault).await?;

        for job in aws_jobs {
            match &*job.action {
                "InventoryRetrieval" => {
                    debug!("InventoryRetrieval job found");

                    match &*job.status_code {
                        "Succeeded" => {
                            let trans = repo.get_transaction().await?;
                            Repository::delete_archive_associations(&trans, &vault).await?;

                            for archive in
                                aws_glacier.get_inventory_job_result(&vault, &job).await?
                            {
                                match Repository::create_archive(&trans, &archive).await {
                                    Err(_) => {
                                        Repository::update_archive(&trans, &archive).await?;
                                    }
                                    _ => {}
                                };
                                Repository::create_archive_association(&trans, &vault, &archive)
                                    .await?;
                            }

                            trans.commit().await?;
                        }
                        status_code => {
                            info!("Job status \"{}\" => skipping job", status_code);
                        }
                    }
                }
                action => {
                    info!("Unkonwn job action found: \"{}\"", action);
                }
            }
        }
    }

    Ok(())
}
