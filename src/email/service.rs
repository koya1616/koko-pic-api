use crate::email::types::{EmailMessage, SmtpConfig};
use anyhow::Result;
use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
  Message, Tokio1Executor,
};

pub struct EmailService {
  smtp_config: SmtpConfig,
  transporter: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
  pub fn new(smtp_config: SmtpConfig) -> Result<Self> {
    let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());

    let transporter = if smtp_config.host == "localhost" || smtp_config.host == "mailhog" {
      AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_config.host)
        .credentials(creds)
        .port(smtp_config.port)
        .build()
    } else {
      AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_config.host)?
        .credentials(creds)
        .port(smtp_config.port)
        .build()
    };

    Ok(EmailService {
      smtp_config,
      transporter,
    })
  }

  pub async fn send_email(&self, message: &EmailMessage) -> Result<()> {
    for recipient in &message.to {
      let email = Message::builder()
        .from(self.smtp_config.from_email.parse()?)
        .to(recipient.parse()?)
        .subject(&message.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(message.body.clone())?;

      self.transporter.send(email).await?;
    }

    Ok(())
  }

  pub async fn send_simple_text_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
    let message = EmailMessage::new(vec![to.to_string()], subject.to_string(), body.to_string());
    self.send_email(&message).await
  }

  pub fn build_verification_email_body(token: &str) -> String {
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:1420".to_string());
    let verification_url = format!("{}/verify-email/{}", frontend_url, token);

    format!(
      "こんにちは、\n\n以下のリンクをクリックしてメールアドレスを確認してください:\n\n{}\n\nこのリンクは24時間有効です。\n\nよろしくお願いします。",
      verification_url
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  #[tokio::test]
  #[ignore]
  async fn test_send_email() -> Result<()> {
    dotenvy::dotenv().ok();

    let smtp_config = SmtpConfig {
      host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
      port: env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap(),
      username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME environment variable must be set."),
      password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD environment variable must be set."),
      from_email: env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL environment variable must be set."),
    };

    let email_service = EmailService::new(smtp_config)?;

    let message = EmailMessage::new(
      vec!["test@example.com".to_string()],
      "Test Subject".to_string(),
      "Test Body".to_string(),
    );

    let result = email_service.send_email(&message).await;
    assert!(result.is_ok());

    Ok(())
  }

  #[tokio::test]
  #[ignore]
  async fn test_send_simple_text_email() -> Result<()> {
    dotenvy::dotenv().ok();

    let smtp_config = SmtpConfig {
      host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
      port: env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap(),
      username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME environment variable must be set."),
      password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD environment variable must be set."),
      from_email: env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL environment variable must be set."),
    };

    let email_service = EmailService::new(smtp_config)?;

    let result = email_service
      .send_simple_text_email("test@example.com", "Simple Test Subject", "Simple Test Body")
      .await;
    assert!(result.is_ok());

    Ok(())
  }

  #[test]
  fn test_build_verification_email_body() {
    env::set_var("FRONTEND_URL", "https://example.com");

    let token = "abc123";
    let expected_body = "こんにちは、\n\n以下のリンクをクリックしてメールアドレスを確認してください:\n\nhttps://example.com/verify-email/abc123\n\nこのリンクは24時間有効です。\n\nよろしくお願いします。";

    let actual_body = EmailService::build_verification_email_body(token);
    assert_eq!(actual_body, expected_body);

    env::remove_var("FRONTEND_URL");
  }

  #[test]
  fn test_build_verification_email_body_default_url() {
    env::remove_var("FRONTEND_URL");

    let token = "def456";
    let expected_body = "こんにちは、\n\n以下のリンクをクリックしてメールアドレスを確認してください:\n\nhttp://localhost:1420/verify-email/def456\n\nこのリンクは24時間有効です。\n\nよろしくお願いします。";

    let actual_body = EmailService::build_verification_email_body(token);
    assert_eq!(actual_body, expected_body);
  }

  #[tokio::test]
  async fn test_email_service_new_with_localhost_smtp() -> Result<()> {
    let smtp_config = SmtpConfig {
      host: "localhost".to_string(),
      port: 1025,
      username: "test_user".to_string(),
      password: "test_password".to_string(),
      from_email: "test@example.com".to_string(),
    };

    let email_service = EmailService::new(smtp_config)?;
    assert_eq!(email_service.smtp_config.host, "localhost");
    assert_eq!(email_service.smtp_config.port, 1025);

    Ok(())
  }

  #[tokio::test]
  async fn test_email_service_new_with_remote_smtp() -> Result<()> {
    let smtp_config = SmtpConfig {
      host: "smtp.example.com".to_string(),
      port: 587,
      username: "test_user".to_string(),
      password: "test_password".to_string(),
      from_email: "test@example.com".to_string(),
    };

    let email_service = EmailService::new(smtp_config)?;
    assert_eq!(email_service.smtp_config.host, "smtp.example.com");
    assert_eq!(email_service.smtp_config.port, 587);

    Ok(())
  }
}
