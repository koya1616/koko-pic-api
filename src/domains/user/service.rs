use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::error::Error;
use validator::Validate;

use super::{
  model::{CreateUserRequest, LoginRequest, LoginResponse, User},
  repository::{UserRepository, VerificationTokenRepository},
};
use crate::{
  email::EmailService,
  utils::jwt::{encode_jwt, Claims},
};

#[derive(Debug)]
pub enum UserServiceError {
  Unauthorized,
  ValidationError,
  InternalServerError,
  InvalidToken,
  TokenExpired,
  TokenAlreadyUsed,
  UserNotFound,
}

impl Error for UserServiceError {}

impl std::fmt::Display for UserServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UserServiceError::Unauthorized => write!(f, "Unauthorized"),
      UserServiceError::ValidationError => write!(f, "Validation Error"),
      UserServiceError::InternalServerError => write!(f, "Internal Server Error"),
      UserServiceError::InvalidToken => write!(f, "Invalid Token"),
      UserServiceError::TokenExpired => write!(f, "Token Expired"),
      UserServiceError::TokenAlreadyUsed => write!(f, "Token Already Used"),
      UserServiceError::UserNotFound => write!(f, "User Not Found"),
    }
  }
}

impl From<sqlx::Error> for UserServiceError {
  fn from(_: sqlx::Error) -> Self {
    UserServiceError::InternalServerError
  }
}

#[async_trait]
pub trait UserService: Send + Sync {
  async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserServiceError>;
  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError>;
  async fn send_verification_email(&self, user_id: i32) -> Result<(), UserServiceError>;
  async fn verify_email(&self, token: String) -> Result<User, UserServiceError>;
}

pub struct UserServiceImpl<U, V> {
  user_repository: U,
  verification_token_repository: V,
  email_service: EmailService,
}

impl<U, V> UserServiceImpl<U, V>
where
  U: UserRepository,
  V: VerificationTokenRepository,
{
  pub fn new(user_repository: U, verification_token_repository: V, email_service: EmailService) -> Self {
    Self {
      user_repository,
      verification_token_repository,
      email_service,
    }
  }
}

#[async_trait]
impl<U, V> UserService for UserServiceImpl<U, V>
where
  U: UserRepository,
  V: VerificationTokenRepository,
{
  async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserServiceError> {
    req.validate().map_err(|_| UserServiceError::ValidationError)?;

    let user = self
      .user_repository
      .create(&req.email, &req.display_name, &req.password)
      .await?;

    self.send_verification_email(user.id).await?;

    Ok(user)
  }

  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError> {
    let user = self.user_repository.find_by_email(&req.email).await?;

    let user = match user {
      Some(user) => user,
      None => return Err(UserServiceError::Unauthorized),
    };

    if !user.email_verified {
      return Err(UserServiceError::Unauthorized);
    }

    let hashed_input_password = crate::utils::hash_password(&req.password);
    if user.password != hashed_input_password {
      return Err(UserServiceError::Unauthorized);
    }

    let expiration = Utc::now()
      .checked_add_signed(Duration::hours(24))
      .expect("Valid timestamp")
      .timestamp() as usize;

    let claims = Claims {
      sub: user.email.clone(),
      exp: expiration,
      user_id: user.id,
    };

    let token = encode_jwt(claims).map_err(|_| UserServiceError::InternalServerError)?;

    Ok(LoginResponse {
      token,
      user_id: user.id,
      email: user.email,
      display_name: user.display_name,
    })
  }

  async fn send_verification_email(&self, user_id: i32) -> Result<(), UserServiceError> {
    let expires_at = Utc::now()
      .checked_add_signed(Duration::hours(24))
      .ok_or(UserServiceError::InternalServerError)?;

    let verification_token = self
      .verification_token_repository
      .create_verification_token(user_id, "email_verification", expires_at)
      .await?;

    let user = self
      .user_repository
      .find_by_id(user_id)
      .await?
      .ok_or(UserServiceError::UserNotFound)?;

    let subject = "メールアドレスを確認してください";
    let body = format!(
      "こんにちは、\n\n以下のトークンを使用してメールアドレスを確認してください:\n\n{}\n\nこのトークンは24時間有効です。\n\nよろしくお願いします。",
      verification_token.token
    );

    match self
      .email_service
      .send_simple_text_email(&user.email, subject, &body)
      .await
    {
      Ok(_) => tracing::info!("Verification email sent to user {}", user_id),
      Err(e) => tracing::error!("Failed to send verification email to user {}: {:?}", user_id, e),
    }

    Ok(())
  }

  async fn verify_email(&self, token: String) -> Result<User, UserServiceError> {
    let verification_token = self
      .verification_token_repository
      .find_token_by_value(&token)
      .await?
      .ok_or(UserServiceError::InvalidToken)?;

    if verification_token.expires_at < Utc::now() {
      return Err(UserServiceError::TokenExpired);
    }

    if verification_token.used_at.is_some() {
      return Err(UserServiceError::TokenAlreadyUsed);
    }

    let user = User::verify_email(self.user_repository.get_pool(), verification_token.user_id).await?;

    self
      .verification_token_repository
      .mark_token_as_used(verification_token.id)
      .await?;

    Ok(user)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    domains::user::{
      model::{CreateUserRequest, VerificationToken},
      repository::{SqlxUserRepository, SqlxVerificationTokenRepository},
    },
    email::{EmailService, SmtpConfig},
  };
  use sqlx::PgPool;

  fn create_test_email_service() -> EmailService {
    let smtp_config = SmtpConfig {
      host: "localhost".to_string(),
      port: 1025,
      username: "test".to_string(),
      password: "test".to_string(),
      from_email: "noreply@test.com".to_string(),
    };
    EmailService::new(smtp_config).expect("Failed to create test email service")
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn test_create_user_with_verification(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_repo = SqlxUserRepository::new(pool.clone());
    let token_repo = SqlxVerificationTokenRepository::new(pool);
    let email_service = create_test_email_service();

    let service = UserServiceImpl::new(user_repo, token_repo, email_service);

    let req = CreateUserRequest {
      email: "test@example.com".to_string(),
      display_name: "Test User".to_string(),
      password: "password123".to_string(),
    };

    let user = service.create_user(req).await?;

    assert_eq!(user.email, "test@example.com");
    assert!(!user.email_verified);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn test_verify_email(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::create(&pool, "verify@example.com", "Verify Test", "password123").await?;
    assert!(!user.email_verified);

    let expires_at = Utc::now()
      .checked_add_signed(Duration::hours(24))
      .ok_or("Failed to create expiration time")?;

    let verification_token = VerificationToken::create(&pool, user.id, "email_verification", expires_at).await?;

    let user_repo = SqlxUserRepository::new(pool.clone());
    let token_repo = SqlxVerificationTokenRepository::new(pool);
    let email_service = create_test_email_service();

    let service = UserServiceImpl::new(user_repo, token_repo, email_service);

    let verified_user = service.verify_email(verification_token.token).await?;

    assert!(verified_user.email_verified);

    Ok(())
  }

  #[sqlx::test(migrations = "./migrations")]
  async fn test_login_fails_for_unverified_email(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::create(&pool, "unverified@example.com", "Unverified User", "password123").await?;
    assert!(!user.email_verified);

    let user_repo = SqlxUserRepository::new(pool.clone());
    let token_repo = SqlxVerificationTokenRepository::new(pool);
    let email_service = create_test_email_service();
    let service = UserServiceImpl::new(user_repo, token_repo, email_service);

    let login_req = LoginRequest {
      email: "unverified@example.com".to_string(),
      password: "password123".to_string(),
    };

    let result = service.login(login_req).await;
    assert!(matches!(result, Err(UserServiceError::Unauthorized)));

    Ok(())
  }
}
