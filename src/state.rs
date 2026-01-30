use std::sync::Arc;

use sqlx::PgPool;

use crate::{
  domains::{
    picture::{
      model::Picture,
      service::{PictureService, PictureServiceError, PictureServiceImpl},
    },
    request::{
      model::{CreateRequestRequest, Request, RequestsResponse},
      service::RequestService,
    },
    user::{
      model::{CreateUserRequest, LoginRequest, LoginResponse, User, VerifyEmailResponse},
      repository::{SqlxUserRepository, SqlxVerificationTokenRepository},
      service::{UserService, UserServiceError, UserServiceImpl},
    },
  },
  email::EmailService,
  storage::S3Storage,
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
  fn create_picture(
    &self,
    user_id: i32,
    image_url: String,
  ) -> impl std::future::Future<Output = Result<Picture, PictureServiceError>> + Send;
  fn upload_and_create_picture(
    &self,
    user_id: i32,
    file_data: Vec<u8>,
    file_name: String,
    content_type: String,
  ) -> impl std::future::Future<Output = Result<Picture, PictureServiceError>> + Send;
  fn delete_picture(
    &self,
    picture_id: i32,
    user_id: i32,
  ) -> impl std::future::Future<Output = Result<(), PictureServiceError>> + Send;
  fn get_requests(
    &self,
    user_lat: Option<f64>,
    user_lng: Option<f64>,
  ) -> impl std::future::Future<Output = Result<RequestsResponse, sqlx::Error>> + Send;
  fn create_request(
    &self,
    user_id: i32,
    req: CreateRequestRequest,
  ) -> impl std::future::Future<Output = Result<Request, sqlx::Error>> + Send;
  fn get_request_by_id(
    &self,
    request_id: i32,
  ) -> impl std::future::Future<Output = Result<Request, sqlx::Error>> + Send;
}

#[derive(Clone)]
pub struct SharedAppState {
  pub user_service: Arc<UserServiceImpl<SqlxUserRepository, SqlxVerificationTokenRepository>>,
  pub picture_service: Arc<PictureServiceImpl>,
  pub request_service: Arc<RequestService>,
}

impl SharedAppState {
  pub async fn new(pool: PgPool, email_service: EmailService, storage: S3Storage) -> Self {
    let user_repository = SqlxUserRepository::new(pool.clone());
    let verification_token_repository = SqlxVerificationTokenRepository::new(pool.clone());
    let user_service = Arc::new(UserServiceImpl::new(
      user_repository,
      verification_token_repository,
      email_service,
    ));

    let picture_service = Arc::new(PictureServiceImpl::new(pool.clone(), storage));
    let request_service = Arc::new(RequestService::new(pool));

    Self {
      user_service,
      picture_service,
      request_service,
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

  async fn create_picture(&self, user_id: i32, image_url: String) -> Result<Picture, PictureServiceError> {
    self.picture_service.create_picture(user_id, image_url).await
  }

  async fn upload_and_create_picture(
    &self,
    user_id: i32,
    file_data: Vec<u8>,
    file_name: String,
    content_type: String,
  ) -> Result<Picture, PictureServiceError> {
    self
      .picture_service
      .upload_and_create_picture(user_id, file_data, file_name, content_type)
      .await
  }

  async fn delete_picture(&self, picture_id: i32, user_id: i32) -> Result<(), PictureServiceError> {
    self.picture_service.delete_picture(picture_id, user_id).await
  }

  async fn get_requests(&self, user_lat: Option<f64>, user_lng: Option<f64>) -> Result<RequestsResponse, sqlx::Error> {
    self.request_service.get_requests(user_lat, user_lng).await
  }

  async fn create_request(&self, user_id: i32, req: CreateRequestRequest) -> Result<Request, sqlx::Error> {
    self.request_service.create_request(user_id, req).await
  }

  async fn get_request_by_id(&self, request_id: i32) -> Result<Request, sqlx::Error> {
    self.request_service.get_request_by_id(request_id).await
  }
}
