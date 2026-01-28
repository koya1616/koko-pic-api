use anyhow::{Context, Result};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
  config::{Credentials, SharedCredentialsProvider},
  primitives::ByteStream,
  Client as S3Client,
};
use std::env;

#[derive(Clone)]
pub struct S3Storage {
  client: S3Client,
  bucket: String,
  endpoint: Option<String>,
  public_endpoint: Option<String>,
}

impl S3Storage {
  pub async fn new() -> Result<Self> {
    let endpoint = env::var("S3_ENDPOINT").ok();
    let public_endpoint = env::var("S3_PUBLIC_ENDPOINT").ok();
    let access_key = env::var("S3_ACCESS_KEY").context("S3_ACCESS_KEY not set")?;
    let secret_key = env::var("S3_SECRET_KEY").context("S3_SECRET_KEY not set")?;
    let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let bucket = env::var("S3_BUCKET").context("S3_BUCKET not set")?;

    let credentials = Credentials::new(access_key, secret_key, None, None, "custom");
    let credentials_provider = SharedCredentialsProvider::new(credentials);

    let mut config_builder = aws_config::defaults(BehaviorVersion::latest())
      .region(Region::new(region))
      .credentials_provider(credentials_provider);

    if let Some(ref endpoint_url) = endpoint {
      config_builder = config_builder.endpoint_url(endpoint_url);
    }

    let config = config_builder.load().await;
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&config).force_path_style(true);

    if let Some(ref endpoint_url) = endpoint {
      s3_config_builder = s3_config_builder.endpoint_url(endpoint_url);
    }

    let client = S3Client::from_conf(s3_config_builder.build());

    Ok(Self {
      client,
      bucket,
      endpoint,
      public_endpoint,
    })
  }

  pub async fn upload_file(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<String> {
    let byte_stream = ByteStream::from(data);

    self
      .client
      .put_object()
      .bucket(&self.bucket)
      .key(key)
      .body(byte_stream)
      .content_type(content_type)
      .send()
      .await
      .map_err(|e| anyhow::anyhow!("Failed to upload file to S3: {:?}", e))?;

    let endpoint_for_url = self.public_endpoint.as_ref().or(self.endpoint.as_ref());

    let url = if let Some(endpoint) = endpoint_for_url {
      format!("{}/{}/{}", endpoint, self.bucket, key)
    } else {
      format!("https://{}.s3.amazonaws.com/{}", self.bucket, key)
    };

    Ok(url)
  }
}
