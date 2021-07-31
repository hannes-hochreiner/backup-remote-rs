extern crate backup_remote_rs;
use anyhow::Result;
use backup_remote_rs::aws_glacier::AwsGlacier;
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
        .get_matches();

    // Update list of vaults
    let aws_glacier = AwsGlacier::new(
        matches.value_of("secret_key").unwrap(),
        matches.value_of("key_id").unwrap(),
        matches.value_of("region").unwrap(),
    );

    let vault_list = aws_glacier.list_vaults().await?;

    println!("{:?}", vault_list);
    // If the inventory of a vault is older than 1 week, launch an inventory job

    // Check inventory jobs and launch workers as needed

    // Check download jobs and launch workers as needed

    // Check upload jobs and launch workers as needed

    Ok(())
}
