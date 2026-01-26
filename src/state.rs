use std::sync::Arc;

use sqlx::PgPool;

use crate::domains::user::{
  model::{CreateUserRequest, LoginRequest, LoginResponse, User},
  repository::SqlxUserRepository,
  service::{UserService, UserServiceError, UserServiceImpl},
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
}

#[derive(Clone)]
pub struct SharedAppState {
  pub user_service: Arc<UserServiceImpl<SqlxUserRepository>>,
}

impl SharedAppState {
  pub async fn new(pool: PgPool) -> Self {
    let user_repository = SqlxUserRepository::new(pool);
    let user_service = Arc::new(UserServiceImpl::new(user_repository));

    Self { user_service }
  }
}

impl AppState for SharedAppState {
  async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserServiceError> {
    self.user_service.create_user(req).await
  }

  async fn login(&self, req: LoginRequest) -> Result<LoginResponse, UserServiceError> {
    self.user_service.login(req).await
  }
}
