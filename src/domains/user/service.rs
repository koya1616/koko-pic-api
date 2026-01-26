use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use validator::Validate;

use super::{
  model::{CreateUserRequest, LoginRequest, LoginResponse, User},
  repository::UserRepository,
};

#[derive(Debug)]
pub enum UserServiceError {
  Unauthorized,
  ValidationError,
  InternalServerError,
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
}

pub struct UserServiceImpl<R> {
  repository: R,
}

impl<R> UserServiceImpl<R>
where
  R: UserRepository,
{
  pub fn new(repository: R) -> Self {
    Self { repository }
  }
}

#[async_trait]
impl<R> UserService for UserServiceImpl<R>
where
  R: UserRepository,
{
  async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserServiceError> {
    req.validate().map_err(|_| UserServiceError::ValidationError)?;

    let user = self
      .repository
      .create(&req.email, &req.display_name, &req.password)
      .await?;
    Ok(user)
  }

  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError> {
    let user = self.repository.find_by_email(&req.email).await?;

    let user = match user {
      Some(user) => user,
      None => return Err(UserServiceError::Unauthorized),
    };

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

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
      .map_err(|_| UserServiceError::InternalServerError)?;

    Ok(LoginResponse {
      token,
      user_id: user.id,
      email: user.email,
      display_name: user.display_name,
    })
  }
}

#[derive(Debug, serde::Serialize)]
struct Claims {
  sub: String,
  exp: usize,
  user_id: i32,
}
