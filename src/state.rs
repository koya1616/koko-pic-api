use std::sync::Arc;

use sqlx::PgPool;

use crate::{
  domains::{
    picture::{
      model::PicturesResponse,
      service::{PictureService, PictureServiceError, PictureServiceImpl},
    },
    user::{
      model::{CreateUserRequest, LoginRequest, LoginResponse, User, VerifyEmailResponse},
      repository::{SqlxUserRepository, SqlxVerificationTokenRepository},
      service::{UserService, UserServiceError, UserServiceImpl},
    },
  },
  email::EmailService,
};

pub trait AppState: Clone + Send + Sync + 'static {
  fn create_user(
    &self,
    req: CreateUserRequest,
  ) -> impl std::future::Future<Output = Result<User, UserServiceError>> + Send;
  fn login(
    &self,
    req: LoginRequest,
  ) -> impl std::future::Future<Output = Result<LoginResponse, UserServiceError>> + Send;
  fn verify_email(
    &self,
    token: String,
  ) -> impl std::future::Future<Output = Result<VerifyEmailResponse, UserServiceError>> + Send;
  fn send_verification_email(
    &self,
    user_id: i32,
  ) -> impl std::future::Future<Output = Result<(), UserServiceError>> + Send;
  fn send_verification_email_by_email(
    &self,
    email: String,
  ) -> impl std::future::Future<Output = Result<(), UserServiceError>> + Send;
  fn get_user_by_id(&self, user_id: i32) -> impl std::future::Future<Output = Result<User, UserServiceError>> + Send;
  fn get_pictures(&self) -> impl std::future::Future<Output = Result<PicturesResponse, PictureServiceError>> + Send;
}

#[derive(Clone)]
pub struct SharedAppState {
  pub user_service: Arc<UserServiceImpl<SqlxUserRepository, SqlxVerificationTokenRepository>>,
  pub picture_service: Arc<PictureServiceImpl>,
}

impl SharedAppState {
  pub async fn new(pool: PgPool, email_service: EmailService) -> Self {
    let user_repository = SqlxUserRepository::new(pool.clone());
    let verification_token_repository = SqlxVerificationTokenRepository::new(pool.clone());
    let user_service = Arc::new(UserServiceImpl::new(
      user_repository,
      verification_token_repository,
      email_service,
    ));

    let picture_service = Arc::new(PictureServiceImpl::new(pool));

    Self {
      user_service,
      picture_service,
    }
  }
}

impl AppState for SharedAppState {
  async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserServiceError> {
    self.user_service.create_user(req).await
  }

  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError> {
    self.user_service.login(req).await
  }

  async fn verify_email(&self, token: String) -> Result<VerifyEmailResponse, UserServiceError> {
    self.user_service.verify_email(token).await
  }

  async fn send_verification_email(&self, user_id: i32) -> Result<(), UserServiceError> {
    self.user_service.send_verification_email(user_id).await
  }

  async fn send_verification_email_by_email(&self, email: String) -> Result<(), UserServiceError> {
    self.user_service.send_verification_email_by_email(email).await
  }

  async fn get_user_by_id(&self, user_id: i32) -> Result<User, UserServiceError> {
    self.user_service.get_user_by_id(user_id).await
  }

  async fn get_pictures(&self) -> Result<PicturesResponse, PictureServiceError> {
    self.picture_service.get_pictures().await
  }
}
