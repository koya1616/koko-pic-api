use regex::Regex;
use sha2::{Digest, Sha256};
use validator::ValidationError;

pub mod error;
pub mod jwt;

pub fn hash_password(password: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(password.as_bytes());
  let result = hasher.finalize();
  format!("{:x}", result)
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
  let letter_regex = Regex::new(r"[a-zA-Z]").unwrap();
  let digit_regex = Regex::new(r"\d").unwrap();

  if !letter_regex.is_match(password) {
    return Err(ValidationError::new("パスワードには英字を含める必要があります"));
  }

  if !digit_regex.is_match(password) {
    return Err(ValidationError::new("パスワードには数字を含める必要があります"));
  }

  Ok(())
}

pub async fn init_email_service() -> anyhow::Result<crate::email::EmailService> {
  use crate::email::{EmailService, SmtpConfig};
  use std::env;

  let smtp_config = SmtpConfig {
    host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
    port: env::var("SMTP_PORT")
      .unwrap_or_else(|_| "587".to_string())
      .parse()
      .unwrap_or(587),
    username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME environment variable must be set."),
    password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD environment variable must be set."),
    from_email: env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL environment variable must be set."),
  };

  let email_service = EmailService::new(smtp_config)?;
  Ok(email_service)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_password_valid() {
    assert!(validate_password("password123").is_ok());
    assert!(validate_password("Test123").is_ok());
    assert!(validate_password("abc123def").is_ok());
    assert!(validate_password("A1").is_ok());
  }

  #[test]
  fn test_validate_password_missing_letter() {
    let result = validate_password("12345678");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("パスワードには英字を含める必要があります"));
  }

  #[test]
  fn test_validate_password_missing_digit() {
    let result = validate_password("abcdefghijk");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("パスワードには数字を含める必要があります"));
  }

  #[test]
  fn test_validate_password_empty() {
    let result = validate_password("");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("パスワードには英字を含める必要があります"));
  }

  #[test]
  fn test_validate_password_only_special_chars() {
    let result = validate_password("!@#$%^&*()");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("パスワードには英字を含める必要があります"));
  }
}
