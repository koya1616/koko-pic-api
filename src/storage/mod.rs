use anyhow::{Context, Result};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
  config::{Credentials, SharedCredentialsProvider},
  primitives::ByteStream,
  Client as S3Client,
};
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
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
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&config);

    let is_supabase = endpoint.as_ref().map(|e| e.contains("supabase")).unwrap_or(false);

    if is_supabase {
      let http_client = HyperClientBuilder::new().build_https();
      s3_config_builder = s3_config_builder.http_client(http_client).force_path_style(true);
    } else {
      s3_config_builder = s3_config_builder.force_path_style(true);
    }

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

  pub async fn delete_file(&self, key: &str) -> Result<()> {
    self
      .client
      .delete_object()
      .bucket(&self.bucket)
      .key(key)
      .send()
      .await
      .map_err(|e| anyhow::anyhow!("Failed to delete file from S3: {:?}", e))?;

    Ok(())
  }

  pub fn extract_key_from_url(&self, url: &str) -> Option<String> {
    let endpoint_for_url = self.public_endpoint.as_ref().or(self.endpoint.as_ref());

    if let Some(endpoint) = endpoint_for_url {
      let prefix = format!("{}/{}/", endpoint, self.bucket);
      if url.starts_with(&prefix) {
        return Some(url[prefix.len()..].to_string());
      }
    } else {
      let prefix = format!("https://{}.s3.amazonaws.com/", self.bucket);
      if url.starts_with(&prefix) {
        return Some(url[prefix.len()..].to_string());
      }
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_storage(endpoint: Option<String>, public_endpoint: Option<String>, bucket: &str) -> S3Storage {
    let credentials = Credentials::new("test_key", "test_secret", None, None, "test");
    let credentials_provider = SharedCredentialsProvider::new(credentials);

    let config_builder = aws_config::defaults(BehaviorVersion::latest())
      .region(Region::new("us-east-1"))
      .credentials_provider(credentials_provider);

    let config = tokio::runtime::Runtime::new().unwrap().block_on(config_builder.load());

    let s3_config_builder = aws_sdk_s3::config::Builder::from(&config);
    let client = S3Client::from_conf(s3_config_builder.build());

    S3Storage {
      client,
      bucket: bucket.to_string(),
      endpoint,
      public_endpoint,
    }
  }

  #[test]
  fn test_extract_key_from_url_with_endpoint() {
    let storage = create_test_storage(Some("http://localhost:9000".to_string()), None, "dev");

    let url = "http://localhost:9000/dev/pictures/test.jpg";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, Some("pictures/test.jpg".to_string()));
  }

  #[test]
  fn test_extract_key_from_url_with_public_endpoint() {
    let storage = create_test_storage(
      Some("http://rustfs:9000".to_string()),
      Some("http://127.0.0.1:9000".to_string()),
      "dev",
    );

    let url = "http://127.0.0.1:9000/dev/pictures/test.jpg";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, Some("pictures/test.jpg".to_string()));
  }

  #[test]
  fn test_extract_key_from_url_with_supabase() {
    let storage = create_test_storage(
      Some("https://project.storage.supabase.co/storage/v1/s3".to_string()),
      None,
      "koko-pic",
    );

    let url = "https://project.storage.supabase.co/storage/v1/s3/koko-pic/pictures/7/test.jpg";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, Some("pictures/7/test.jpg".to_string()));
  }

  #[test]
  fn test_extract_key_from_url_with_cloudflare_r2() {
    let storage = create_test_storage(
      Some("https://3eee0f3be0c1d2517ddd0a5acd4486e7.r2.cloudflarestorage.com".to_string()),
      None,
      "koko-pic",
    );

    let url = "https://3eee0f3be0c1d2517ddd0a5acd4486e7.r2.cloudflarestorage.com/koko-pic/pictures/test.jpg";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, Some("pictures/test.jpg".to_string()));
  }

  #[test]
  fn test_extract_key_from_url_aws_s3() {
    let storage = create_test_storage(None, None, "my-bucket");

    let url = "https://my-bucket.s3.amazonaws.com/uploads/image.png";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, Some("uploads/image.png".to_string()));
  }

  #[test]
  fn test_extract_key_from_url_invalid() {
    let storage = create_test_storage(Some("http://localhost:9000".to_string()), None, "dev");

    let url = "http://example.com/invalid/path.jpg";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, None);
  }

  #[test]
  fn test_extract_key_from_url_wrong_bucket() {
    let storage = create_test_storage(Some("http://localhost:9000".to_string()), None, "dev");

    let url = "http://localhost:9000/production/pictures/test.jpg";
    let key = storage.extract_key_from_url(url);
    assert_eq!(key, None);
  }

  #[tokio::test]
  #[serial_test::serial]
  async fn test_new_with_supabase_endpoint() {
    std::env::set_var("S3_ENDPOINT", "https://test.storage.supabase.co/storage/v1/s3");
    std::env::set_var("S3_ACCESS_KEY", "test_access_key");
    std::env::set_var("S3_SECRET_KEY", "test_secret_key");
    std::env::set_var("S3_REGION", "us-east-1");
    std::env::set_var("S3_BUCKET", "test-bucket");

    let result = S3Storage::new().await;
    assert!(result.is_ok());

    let storage = result.unwrap();
    assert_eq!(storage.bucket, "test-bucket");
    assert_eq!(
      storage.endpoint,
      Some("https://test.storage.supabase.co/storage/v1/s3".to_string())
    );

    std::env::remove_var("S3_ENDPOINT");
    std::env::remove_var("S3_ACCESS_KEY");
    std::env::remove_var("S3_SECRET_KEY");
    std::env::remove_var("S3_REGION");
    std::env::remove_var("S3_BUCKET");
  }

  #[tokio::test]
  #[serial_test::serial]
  async fn test_new_with_cloudflare_r2_endpoint() {
    std::env::set_var("S3_ENDPOINT", "https://abc123.r2.cloudflarestorage.com");
    std::env::set_var("S3_ACCESS_KEY", "test_access_key");
    std::env::set_var("S3_SECRET_KEY", "test_secret_key");
    std::env::set_var("S3_REGION", "auto");
    std::env::set_var("S3_BUCKET", "my-bucket");

    let result = S3Storage::new().await;
    assert!(result.is_ok());

    let storage = result.unwrap();
    assert_eq!(storage.bucket, "my-bucket");
    assert_eq!(
      storage.endpoint,
      Some("https://abc123.r2.cloudflarestorage.com".to_string())
    );

    std::env::remove_var("S3_ENDPOINT");
    std::env::remove_var("S3_ACCESS_KEY");
    std::env::remove_var("S3_SECRET_KEY");
    std::env::remove_var("S3_REGION");
    std::env::remove_var("S3_BUCKET");
  }

  #[tokio::test]
  #[serial_test::serial]
  async fn test_new_with_public_endpoint() {
    std::env::set_var("S3_ENDPOINT", "http://rustfs:9000");
    std::env::set_var("S3_PUBLIC_ENDPOINT", "http://127.0.0.1:9000");
    std::env::set_var("S3_ACCESS_KEY", "test_access_key");
    std::env::set_var("S3_SECRET_KEY", "test_secret_key");
    std::env::set_var("S3_REGION", "us-east-1");
    std::env::set_var("S3_BUCKET", "dev");

    let result = S3Storage::new().await;
    assert!(result.is_ok());

    let storage = result.unwrap();
    assert_eq!(storage.bucket, "dev");
    assert_eq!(storage.endpoint, Some("http://rustfs:9000".to_string()));
    assert_eq!(storage.public_endpoint, Some("http://127.0.0.1:9000".to_string()));

    std::env::remove_var("S3_ENDPOINT");
    std::env::remove_var("S3_PUBLIC_ENDPOINT");
    std::env::remove_var("S3_ACCESS_KEY");
    std::env::remove_var("S3_SECRET_KEY");
    std::env::remove_var("S3_REGION");
    std::env::remove_var("S3_BUCKET");
  }

  #[tokio::test]
  #[serial_test::serial]
  async fn test_new_missing_credentials() {
    std::env::remove_var("S3_ACCESS_KEY");
    std::env::remove_var("S3_SECRET_KEY");
    std::env::remove_var("S3_BUCKET");

    let result = S3Storage::new().await;
    assert!(result.is_err());
  }
}
