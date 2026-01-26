use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub from_email: String,
}

impl Default for SmtpConfig {
  fn default() -> Self {
    SmtpConfig {
      host: "smtp.gmail.com".to_string(),
      port: 587,
      username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
      password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
      from_email: std::env::var("SMTP_FROM_EMAIL").unwrap_or_default(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
  pub to: Vec<String>,
  pub subject: String,
  pub body: String,
}

impl EmailMessage {
  pub fn new(to: Vec<String>, subject: String, body: String) -> Self {
    EmailMessage { to, subject, body }
  }
}
