use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::error::Error;
use validator::Validate;

use super::{
  model::{CreateUserRequest, LoginRequest, LoginResponse, User, VerifyEmailResponse},
  repository::{RepositoryError, UserRepository, VerificationTokenRepository},
};
use crate::{
  email::EmailService,
  utils::jwt::{encode_jwt, Claims},
};

#[derive(Debug)]
pub enum UserServiceError {
  Unauthorized(String),
  ValidationError(String),
  InternalServerError(String),
  InvalidToken(String),
  TokenExpired(String),
  TokenAlreadyUsed(String),
  UserNotFound(String),
}

impl Error for UserServiceError {}

impl std::fmt::Display for UserServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UserServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
      UserServiceError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
      UserServiceError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
      UserServiceError::InvalidToken(msg) => write!(f, "Invalid Token: {}", msg),
      UserServiceError::TokenExpired(msg) => write!(f, "Token Expired: {}", msg),
      UserServiceError::TokenAlreadyUsed(msg) => write!(f, "Token Already Used: {}", msg),
      UserServiceError::UserNotFound(msg) => write!(f, "User Not Found: {}", msg),
    }
  }
}

impl From<RepositoryError> for UserServiceError {
  fn from(err: RepositoryError) -> Self {
    match err {
      RepositoryError::DatabaseError(e) => UserServiceError::InternalServerError(format!("Database error: {}", e)),
      RepositoryError::NotFound(msg) => UserServiceError::UserNotFound(msg),
      RepositoryError::Conflict(msg) => UserServiceError::InternalServerError(msg),
    }
  }
}

impl From<sqlx::Error> for UserServiceError {
  fn from(err: sqlx::Error) -> Self {
    UserServiceError::InternalServerError(format!("Database error: {}", err))
  }
}

#[async_trait]
pub trait UserService: Send + Sync {
  async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserServiceError>;
  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError>;
  async fn send_verification_email(&self, user_id: i32) -> Result<(), UserServiceError>;
  async fn verify_email(&self, token: String) -> Result<VerifyEmailResponse, UserServiceError>;
  async fn get_user_by_id(&self, user_id: i32) -> Result<User, UserServiceError>;
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
    req
      .validate()
      .map_err(|e| UserServiceError::ValidationError(format!("Validation failed: {}", e)))?;

    let user = self
      .user_repository
      .create(&req.email, &req.display_name, &req.password)
      .await?;

    self.send_verification_email(user.id).await?;

    Ok(user)
  }

  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError> {
    let user = self
      .user_repository
      .find_by_email(&req.email)
      .await?
      .ok_or_else(|| UserServiceError::Unauthorized("Invalid credentials".to_string()))?;

    if !user.email_verified {
      return Err(UserServiceError::Unauthorized("Email not verified".to_string()));
    }

    let hashed_input_password = crate::utils::hash_password(&req.password);
    if user.password != hashed_input_password {
      return Err(UserServiceError::Unauthorized("Invalid credentials".to_string()));
    }

    let expiration = Utc::now()
      .checked_add_signed(Duration::hours(24))
      .ok_or_else(|| UserServiceError::InternalServerError("Failed to calculate expiration time".to_string()))?
      .timestamp() as usize;

    let claims = Claims {
      sub: user.email.clone(),
      exp: expiration,
      user_id: user.id,
    };

    let token =
      encode_jwt(claims).map_err(|e| UserServiceError::InternalServerError(format!("JWT encoding failed: {}", e)))?;

    Ok(LoginResponse {
      token,
      user_id: user.id,
      email: user.email,
      display_name: user.display_name,
    })
  }

  async fn send_verification_email(&self, user_id: i32) -> Result<(), UserServiceError> {
    let verification_token = self
      .verification_token_repository
      .create_verification_token(user_id, "email_verification")
      .await?;

    let user = self
      .user_repository
      .find_by_id(user_id)
      .await?
      .ok_or_else(|| UserServiceError::UserNotFound("User not found".to_string()))?;

    let subject = "メールアドレスを確認してください";
    let body = format!(
      "こんにちは、\n\n以下のトークンを使用してメールアドレスを確認してください:\n\n{}\n\nこのトークンは24時間有効です。\n\nよろしくお願いします。",
      verification_token.token
    );

    if let Err(e) = self
      .email_service
      .send_simple_text_email(&user.email, subject, &body)
      .await
    {
      tracing::error!("Failed to send verification email to user {}: {:?}", user_id, e);
    } else {
      tracing::info!("Verification email sent to user {}", user_id);
    }

    Ok(())
  }

  async fn verify_email(&self, token: String) -> Result<VerifyEmailResponse, UserServiceError> {
    let verification_token = self
      .verification_token_repository
      .find_token_by_value(&token)
      .await?
      .ok_or_else(|| UserServiceError::InvalidToken("Invalid verification token".to_string()))?;

    if verification_token.expires_at < Utc::now() {
      return Err(UserServiceError::TokenExpired(
        "Verification token has expired".to_string(),
      ));
    }

    if verification_token.used_at.is_some() {
      return Err(UserServiceError::TokenAlreadyUsed(
        "Verification token has already been used".to_string(),
      ));
    }

    let user = User::verify_email(self.user_repository.get_pool(), verification_token.user_id).await?;

    self
      .verification_token_repository
      .mark_token_as_used(verification_token.id)
      .await?;

    let expiration = Utc::now()
      .checked_add_signed(Duration::hours(24))
      .ok_or_else(|| UserServiceError::InternalServerError("Failed to calculate expiration time".to_string()))?
      .timestamp() as usize;

    let claims = Claims {
      sub: user.email.clone(),
      exp: expiration,
      user_id: user.id,
    };

    let token =
      encode_jwt(claims).map_err(|e| UserServiceError::InternalServerError(format!("JWT encoding failed: {}", e)))?;

    Ok(VerifyEmailResponse {
      token,
      user_id: user.id,
      email: user.email,
      display_name: user.display_name,
    })
  }

  async fn get_user_by_id(&self, user_id: i32) -> Result<User, UserServiceError> {
    let user = self
      .user_repository
      .find_by_id(user_id)
      .await?
      .ok_or_else(|| UserServiceError::UserNotFound("User not found".to_string()))?;

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

    let verification_token = VerificationToken::create(&pool, user.id, "email_verification").await?;

    let user_repo = SqlxUserRepository::new(pool.clone());
    let token_repo = SqlxVerificationTokenRepository::new(pool.clone());
    let email_service = create_test_email_service();

    let service = UserServiceImpl::new(user_repo, token_repo, email_service);

    let verify_response = service.verify_email(verification_token.token).await?;

    assert_eq!(verify_response.user_id, user.id);
    assert_eq!(verify_response.email, user.email);
    assert_eq!(verify_response.display_name, user.display_name);
    assert!(!verify_response.token.is_empty());

    let updated_user = User::find_by_id(&pool, user.id).await?.unwrap();
    assert!(updated_user.email_verified);

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
    assert!(matches!(result, Err(UserServiceError::Unauthorized(_))));

    Ok(())
  }
}
