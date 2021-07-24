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
        .get_matches();

    let secret_key = String::from(matches.value_of("secret_key").unwrap());
    let key_id = String::from(matches.value_of("key_id").unwrap());
    let region = String::from(matches.value_of("region").unwrap());

    match matches.subcommand {
        Some(subcommand) => match &*subcommand.name {
            "list-vaults" => list_vaults(&secret_key, &key_id, &region).await,
            _ => Err(anyhow::Error::msg("unexpected subcommand")),
        },
        None => Err(anyhow::Error::msg("no subcommand found")),
    }
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn sha_256_hash_1() {
        assert_eq!(
            sha_256_hash(&[]).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hash_request_1() {
        assert_eq!(
            hash_request(
                "PUT",
                &"https://glacier.us-east-1.amazonaws.com/-/vaults/examplevault"
                    .parse::<Uri>()
                    .unwrap(),
                &Utc.ymd(2012, 5, 25).and_hms(0, 24, 53),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            )
            .unwrap(),
            "5f1da1a2d0feb614dd03d71e87928b8e449ac87614479332aced3a701f916743"
        );
    }

    #[test]
    fn signature_1() {
        assert_eq!(
            signature(
                "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
                &Utc.ymd(2012, 5, 25).and_hms(0, 24, 53),
                "us-east-1",
                "5f1da1a2d0feb614dd03d71e87928b8e449ac87614479332aced3a701f916743"
            )
            .unwrap(),
            "3ce5b2f2fffac9262b4da9256f8d086b4aaf42eba5f111c21681a65a127b7c2a"
        );
    }
}
