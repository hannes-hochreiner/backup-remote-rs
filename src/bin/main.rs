use anyhow::Result;
use chrono::{DateTime, Utc};
use data_encoding::HEXLOWER;
use hyper::body::HttpBody as _;
use hyper::Uri;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use ring::{digest, hmac};
use std::env;
use tokio::io::{stdout, AsyncWriteExt as _};
extern crate clap;
use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("backup-remote-rs")
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
        .subcommand(SubCommand::with_name("list-vaults").about("list all vaults"))
        .subcommand(
            SubCommand::with_name("init-inventory")
                .about("initiate the inventory retrieval")
                .arg(
                    Arg::with_name("vault_name")
                        .required(true)
                        .long("vault_name")
                        .takes_value(true)
                        .multiple(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("list-jobs")
                .about("list jobs for a vault")
                .arg(
                    Arg::with_name("vault_name")
                        .required(true)
                        .long("vault_name")
                        .takes_value(true)
                        .multiple(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("job-output")
                .about("get the output of a job")
                .arg(
                    Arg::with_name("vault_name")
                        .required(true)
                        .long("vault_name")
                        .takes_value(true)
                        .multiple(false),
                )
                .arg(
                    Arg::with_name("job_id")
                        .required(true)
                        .long("job_id")
                        .takes_value(true)
                        .multiple(false),
                ),
        )
        .get_matches();

    let secret_key = String::from(matches.value_of("secret_key").unwrap());
    let key_id = String::from(matches.value_of("key_id").unwrap());
    let region = String::from(matches.value_of("region").unwrap());

    match matches.subcommand {
        Some(subcommand) => match &*subcommand.name {
            "list-vaults" => list_vaults(&secret_key, &key_id, &region).await,
            "init-inventory" => {
                init_inventory_retrieval(
                    &secret_key,
                    &key_id,
                    &region,
                    subcommand.matches.value_of("vault_name").unwrap(),
                )
                .await
            }
            "list-jobs" => {
                list_jobs(
                    &secret_key,
                    &key_id,
                    &region,
                    subcommand.matches.value_of("vault_name").unwrap(),
                )
                .await
            }
            "job-output" => {
                job_output(
                    &secret_key,
                    &key_id,
                    &region,
                    subcommand.matches.value_of("vault_name").unwrap(),
                    subcommand.matches.value_of("job_id").unwrap(),
                )
                .await
            }
            _ => Err(anyhow::Error::msg("unexpected subcommand")),
        },
        None => Err(anyhow::Error::msg("no subcommand found")),
    }
}

async fn job_output(
    secret_key: &str,
    key_id: &str,
    region: &str,
    vault_name: &str,
    job_id: &str,
) -> Result<()> {
    let http_method = "GET";
    let body = "";
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let date_time = Utc::now();
    let uri = format!(
        "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs/{}/output",
        region, vault_name, job_id
    )
    .parse::<Uri>()?;
    let hash_body = sha_256_hash(body.as_bytes())?;
    let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
    let signature = signature(&*secret_key, &date_time, &*region, &*hash_request)?;
    let req = Request::builder()
        .method(http_method)
        .uri(uri)
        .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", key_id, date_time.format("%Y%m%d"), region, signature))
        .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
        .header("x-amz-glacier-version", "2012-06-01")
        .body(Body::from(body))?;
    let mut resp = client.request(req).await?;

    println!("Response: {}", resp.status());

    while let Some(chunk) = resp.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}

async fn list_jobs(secret_key: &str, key_id: &str, region: &str, vault_name: &str) -> Result<()> {
    let http_method = "GET";
    let body = "";
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let date_time = Utc::now();
    let uri = format!(
        "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs",
        region, vault_name
    )
    .parse::<Uri>()?;
    let hash_body = sha_256_hash(body.as_bytes())?;
    let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
    let signature = signature(&*secret_key, &date_time, &*region, &*hash_request)?;
    let req = Request::builder()
        .method(http_method)
        .uri(uri)
        .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", key_id, date_time.format("%Y%m%d"), region, signature))
        .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
        .header("x-amz-glacier-version", "2012-06-01")
        .body(Body::from(body))?;
    let mut resp = client.request(req).await?;

    println!("Response: {}", resp.status());

    while let Some(chunk) = resp.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}

async fn init_inventory_retrieval(
    secret_key: &str,
    key_id: &str,
    region: &str,
    vault_name: &str,
) -> Result<()> {
    let http_method = "POST";
    let body = format!("{{\"Type\": \"inventory-retrieval\", \"Description\": \"{} inventory job\", \"Format\": \"JSON\"}}", vault_name);
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let date_time = Utc::now();
    let uri = format!(
        "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs",
        region, vault_name
    )
    .parse::<Uri>()?;
    let hash_body = sha_256_hash(body.as_bytes())?;
    let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
    let signature = signature(&*secret_key, &date_time, &*region, &*hash_request)?;
    let req = Request::builder()
        .method(http_method)
        .uri(uri)
        .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", key_id, date_time.format("%Y%m%d"), region, signature))
        .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
        .header("x-amz-glacier-version", "2012-06-01")
        .body(Body::from(body))?;
    let mut resp = client.request(req).await?;

    println!("Response: {}", resp.status());

    while let Some(chunk) = resp.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}

async fn list_vaults(secret_key: &str, key_id: &str, region: &str) -> Result<()> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let date_time = Utc::now();
    let uri = format!("https://glacier.{}.amazonaws.com/-/vaults", region).parse::<Uri>()?;
    let hash_body = sha_256_hash(&[])?;
    let hash_request = hash_request("GET", &uri, &date_time, &*hash_body)?;
    let signature = signature(&*secret_key, &date_time, &*region, &*hash_request)?;
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", key_id, date_time.format("%Y%m%d"), region, signature))
        .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
        .header("x-amz-glacier-version", "2012-06-01")
        .body(Body::from(""))?;
    let mut resp = client.request(req).await?;

    println!("Response: {}", resp.status());

    while let Some(chunk) = resp.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}

fn sha_256_hash(data: &[u8]) -> Result<String> {
    Ok(HEXLOWER.encode(digest::digest(&digest::SHA256, data).as_ref()))
}

fn hash_request(
    verb: &str,
    uri: &Uri,
    date_time: &DateTime<Utc>,
    payload_hash: &str,
) -> Result<String> {
    let req = format!("{}\n{}\n\nhost:{}\nx-amz-date:{}\nx-amz-glacier-version:2012-06-01\n\nhost;x-amz-date;x-amz-glacier-version\n{}", verb, uri.path(), uri.host().unwrap(), date_time.format("%Y%m%dT%H%M%SZ"), payload_hash);
    sha_256_hash(req.as_bytes())
}

fn signature(
    secret_key: &str,
    date_time: &DateTime<Utc>,
    region: &str,
    request_hash: &str,
) -> Result<String> {
    let key_date = hmac::sign(
        &hmac::Key::new(hmac::HMAC_SHA256, format!("AWS4{}", secret_key).as_bytes()),
        date_time.format("%Y%m%d").to_string().as_bytes(),
    );
    let key_region = hmac::sign(
        &hmac::Key::new(hmac::HMAC_SHA256, key_date.as_ref()),
        region.as_bytes(),
    );
    let key_glacier = hmac::sign(
        &hmac::Key::new(hmac::HMAC_SHA256, key_region.as_ref()),
        b"glacier",
    );
    let key_request = hmac::sign(
        &hmac::Key::new(hmac::HMAC_SHA256, key_glacier.as_ref()),
        b"aws4_request",
    );
    let str_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/glacier/aws4_request\n{}",
        date_time.format("%Y%m%dT%H%M%SZ"),
        date_time.format("%Y%m%d"),
        region,
        request_hash
    );

    Ok(HEXLOWER.encode(
        hmac::sign(
            &hmac::Key::new(hmac::HMAC_SHA256, key_request.as_ref()),
            str_to_sign.as_bytes(),
        )
        .as_ref(),
    ))
}
