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
      username: env::var("SMTP_USERNAME")?,
      password: env::var("SMTP_PASSWORD")?,
      from_email: env::var("SMTP_FROM_EMAIL")?,
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
}
