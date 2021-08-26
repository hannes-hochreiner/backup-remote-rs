use super::aws_archive::{AwsArchive, AwsIventoryResponse};
use super::aws_job::{AwsJob, AwsJobListResponse};
use super::aws_vault::{AwsVault, AwsVaultListResponse};
use anyhow::Result;
use chrono::{DateTime, Utc};
use data_encoding::HEXLOWER;
use hyper::Uri;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use log::debug;
use ring::{digest, hmac};

pub struct AwsGlacier {
    secret_key: String,
    key_id: String,
    region: String,
}

impl AwsGlacier {
    pub fn new(secret_key: &str, key_id: &str, region: &str) -> Self {
        AwsGlacier {
            secret_key: secret_key.into(),
            key_id: key_id.into(),
            region: region.into(),
        }
    }

    pub async fn list_vaults(&self) -> Result<Vec<AwsVault>> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let date_time = Utc::now();
        let uri =
            format!("https://glacier.{}.amazonaws.com/-/vaults", self.region).parse::<Uri>()?;
        let hash_body = sha_256_hash(&[])?;
        let hash_request = hash_request("GET", &uri, &date_time, &*hash_body)?;
        let signature = self.signature(&date_time, &*hash_request)?;
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", self.key_id, date_time.format("%Y%m%d"), self.region, signature))
            .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-glacier-version", "2012-06-01")
            .body(Body::from(""))?;
        let resp = client.request(req).await?;

        match resp.status() {
            hyper::StatusCode::OK => {
                let resp_body = hyper::body::to_bytes(resp).await?;
                let resp_json: AwsVaultListResponse = serde_json::from_slice(&resp_body)?;

                Ok(resp_json.vault_list)
            }
            _ => {
                debug!("{}", resp.status());
                Err(anyhow::Error::msg(format!("failed to retrieve vault list (status: {})", resp.status())))
            }
        }
    }

    pub async fn list_jobs_for_vault(&self, vault: &AwsVault) -> Result<Vec<AwsJob>> {
        let http_method = "GET";
        let body = "";
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let date_time = Utc::now();
        let uri = format!(
            "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs",
            self.region, vault.vault_name
        )
        .parse::<Uri>()?;
        let hash_body = sha_256_hash(body.as_bytes())?;
        let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
        let signature = self.signature(&date_time, &*hash_request)?;
        let req = Request::builder()
            .method(http_method)
            .uri(uri)
            .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", self.key_id, date_time.format("%Y%m%d"), self.region, signature))
            .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-glacier-version", "2012-06-01")
            .body(Body::from(body))?;
        let resp = client.request(req).await?;

        match resp.status() {
            hyper::StatusCode::OK => {
                let resp_body = hyper::body::to_bytes(resp).await?;
                let resp_json: AwsJobListResponse = serde_json::from_slice(&resp_body)?;

                Ok(resp_json.job_list)
            }
            _ => Err(anyhow::Error::msg("failed to retrieve vault list")),
        }
    }

    pub async fn init_inventory_job_for_vault(&self, vault: &AwsVault) -> Result<String> {
        let http_method = "POST";
        let body = format!("{{\"Type\": \"inventory-retrieval\", \"Description\": \"backup-remote\", \"Format\": \"JSON\"}}");
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let date_time = Utc::now();
        let uri = format!(
            "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs",
            self.region, vault.vault_name
        )
        .parse::<Uri>()?;
        let hash_body = sha_256_hash(body.as_bytes())?;
        let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
        let signature = self.signature(&date_time, &*hash_request)?;
        let req = Request::builder()
            .method(http_method)
            .uri(uri)
            .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", self.key_id, date_time.format("%Y%m%d"), self.region, signature))
            .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-glacier-version", "2012-06-01")
            .body(Body::from(body))?;
        let resp = client.request(req).await?;

        match resp.status() {
            hyper::StatusCode::OK => Ok(resp.headers()["x-amz-job-id"].to_str()?.into()),
            _ => Err(anyhow::Error::msg(format!(
                "failed to initiate inventory job: {}",
                resp.status()
            ))),
        }
    }

    pub async fn get_job_by_id_vault(&self, vault: &AwsVault, job_id: &str) -> Result<AwsJob> {
        let http_method = "GET";
        let body = String::new();
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let date_time = Utc::now();
        let uri = format!(
            "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs/{}",
            self.region, vault.vault_name, job_id
        )
        .parse::<Uri>()?;
        let hash_body = sha_256_hash(body.as_bytes())?;
        let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
        let signature = self.signature(&date_time, &*hash_request)?;
        let req = Request::builder()
            .method(http_method)
            .uri(uri)
            .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", self.key_id, date_time.format("%Y%m%d"), self.region, signature))
            .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-glacier-version", "2012-06-01")
            .body(Body::from(body))?;
        let resp = client.request(req).await?;

        match resp.status() {
            hyper::StatusCode::OK => {
                let resp_body = hyper::body::to_bytes(resp).await?;
                let resp_json: AwsJob = serde_json::from_slice(&resp_body)?;

                Ok(resp_json)
            }
            _ => Err(anyhow::Error::msg("failed to retrieve vault list")),
        }
    }

    pub async fn get_inventory_job_result(
        &self,
        vault: &AwsVault,
        job: &AwsJob,
    ) -> Result<Vec<AwsArchive>> {
        let http_method = "GET";
        let body = "";
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let date_time = Utc::now();
        let uri = format!(
            "https://glacier.{}.amazonaws.com/-/vaults/{}/jobs/{}/output",
            self.region, vault.vault_name, job.job_id
        )
        .parse::<Uri>()?;
        let hash_body = sha_256_hash(body.as_bytes())?;
        let hash_request = hash_request(http_method, &uri, &date_time, &*hash_body)?;
        let signature = self.signature(&date_time, &*hash_request)?;
        let req = Request::builder()
            .method(http_method)
            .uri(uri)
            .header("Authorization", format!("AWS4-HMAC-SHA256 Credential={}/{}/{}/glacier/aws4_request,SignedHeaders=host;x-amz-date;x-amz-glacier-version,Signature={}", self.key_id, date_time.format("%Y%m%d"), self.region, signature))
            .header("x-amz-date", date_time.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-glacier-version", "2012-06-01")
            .body(Body::from(body))?;
        let resp = client.request(req).await?;

        match resp.status() {
            hyper::StatusCode::OK => {
                let resp_body = hyper::body::to_bytes(resp).await?;
                let resp_json: AwsIventoryResponse = serde_json::from_slice(&resp_body)?;

                Ok(resp_json.archive_list)
            }
            _ => {
                debug!("{}", resp.status());
                Err(anyhow::Error::msg(
                    "failed to retrieve inventory job result",
                ))
            }
        }
    }

    fn signature(&self, date_time: &DateTime<Utc>, request_hash: &str) -> Result<String> {
        let key_date = hmac::sign(
            &hmac::Key::new(
                hmac::HMAC_SHA256,
                format!("AWS4{}", self.secret_key).as_bytes(),
            ),
            date_time.format("%Y%m%d").to_string().as_bytes(),
        );
        let key_region = hmac::sign(
            &hmac::Key::new(hmac::HMAC_SHA256, key_date.as_ref()),
            self.region.as_bytes(),
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
            self.region,
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
        let ag = AwsGlacier::new(
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "key_id",
            "us-east-1",
        );
        let sig = ag
            .signature(
                &Utc.ymd(2012, 5, 25).and_hms(0, 24, 53),
                "5f1da1a2d0feb614dd03d71e87928b8e449ac87614479332aced3a701f916743",
            )
            .unwrap();

        assert_eq!(
            sig,
            "3ce5b2f2fffac9262b4da9256f8d086b4aaf42eba5f111c21681a65a127b7c2a"
        );
    }
}
