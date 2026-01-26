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
      username: "".to_string(),
      password: "".to_string(),
      from_email: "".to_string(),
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
