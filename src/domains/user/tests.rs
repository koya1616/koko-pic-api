#[cfg(test)]
mod tests {
  use crate::domains::user::{
    model::{CreateUserRequest, LoginRequest, User},
    repository::UserRepository,
    service::{UserService, UserServiceError, UserServiceImpl},
  };
  use async_trait::async_trait;
  use chrono::Utc;
  use mockall::{predicate::*, *};
  use tokio;

  mockall::mock! {
      UserRepository {}

      #[async_trait]
      impl UserRepository for UserRepository {
          async fn create(&self, email: &str, display_name: &str, password: &str) -> Result<User, sqlx::Error>;
          async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
      }
  }

  #[tokio::test]
  async fn test_create_user_success() {
    let mut mock_repo = MockUserRepository::new();
    mock_repo
      .expect_create()
      .with(
        predicate::eq("test@example.com"),
        predicate::eq("Test User"),
        predicate::always(), // Password hashing happens internally
      )
      .times(1)
      .returning(|_, _, _| {
        Ok(User {
          id: 1,
          email: "test@example.com".to_string(),
          display_name: "Test User".to_string(),
          password: "hashed_password".to_string(),
          created_at: Some(Utc::now()),
        })
      });

    let service = UserServiceImpl::new(mock_repo);
    let req = CreateUserRequest {
      email: "test@example.com".to_string(),
      display_name: "Test User".to_string(),
      password: "password123".to_string(),
    };

    let result = service.create_user(req).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.display_name, "Test User");
  }

  #[tokio::test]
  async fn test_login_success() {
    let mut mock_repo = MockUserRepository::new();

    // Setup expectation for find_by_email
    mock_repo
      .expect_find_by_email()
      .with(predicate::eq("test@example.com"))
      .times(1)
      .returning(|_| {
        Ok(Some(User {
          id: 1,
          email: "test@example.com".to_string(),
          display_name: "Test User".to_string(),
          password: crate::utils::hash_password("password123"), // Hashed password
          created_at: Some(Utc::now()),
        }))
      });

    let service = UserServiceImpl::new(mock_repo);
    let req = LoginRequest {
      email: "test@example.com".to_string(),
      password: "password123".to_string(),
    };

    let result = service.login(req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.email, "test@example.com");
    assert_eq!(response.display_name, "Test User");
  }

  #[tokio::test]
  async fn test_login_invalid_credentials() {
    let mut mock_repo = MockUserRepository::new();

    // Setup expectation for find_by_email - return None (user not found)
    mock_repo
      .expect_find_by_email()
      .with(predicate::eq("nonexistent@example.com"))
      .times(1)
      .returning(|_| Ok(None));

    let service = UserServiceImpl::new(mock_repo);
    let req = LoginRequest {
      email: "nonexistent@example.com".to_string(),
      password: "wrongpassword".to_string(),
    };

    let result = service.login(req).await;
    assert!(result.is_err());
    match result.err().unwrap() {
      UserServiceError::Unauthorized => (), // Expected
      _ => panic!("Expected Unauthorized error"),
    }
  }

  #[tokio::test]
  async fn test_login_wrong_password() {
    let mut mock_repo = MockUserRepository::new();

    // Setup expectation for find_by_email - return user with different password
    mock_repo
      .expect_find_by_email()
      .with(predicate::eq("test@example.com"))
      .times(1)
      .returning(|_| {
        Ok(Some(User {
          id: 1,
          email: "test@example.com".to_string(),
          display_name: "Test User".to_string(),
          password: crate::utils::hash_password("correctpassword"), // Different password
          created_at: Some(Utc::now()),
        }))
      });

    let service = UserServiceImpl::new(mock_repo);
    let req = LoginRequest {
      email: "test@example.com".to_string(),
      password: "wrongpassword".to_string(), // Wrong password
    };

    let result = service.login(req).await;
    assert!(result.is_err());
    match result.err().unwrap() {
      UserServiceError::Unauthorized => (), // Expected
      _ => panic!("Expected Unauthorized error"),
    }
  }
}
