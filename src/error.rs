/// Macro to generate common From implementations for service errors
///
/// Usage:
/// ```ignore
/// impl_service_error_conversions!(UserServiceError, InternalServerError);
/// ```
#[macro_export]
macro_rules! impl_service_error_conversions {
  ($error_type:ty, $internal_variant:ident) => {
    impl From<sqlx::Error> for $error_type {
      fn from(err: sqlx::Error) -> Self {
        <$error_type>::$internal_variant(format!("Database error: {}", err))
      }
    }
  };

  ($error_type:ty, $internal_variant:ident, $not_found_variant:ident) => {
    impl From<sqlx::Error> for $error_type {
      fn from(err: sqlx::Error) -> Self {
        <$error_type>::$internal_variant(format!("Database error: {}", err))
      }
    }

    impl From<$crate::domains::user::repository::RepositoryError> for $error_type {
      fn from(err: $crate::domains::user::repository::RepositoryError) -> Self {
        use $crate::domains::user::repository::RepositoryError;
        match err {
          RepositoryError::DatabaseError(e) => <$error_type>::$internal_variant(format!("Database error: {}", e)),
          RepositoryError::NotFound(msg) => <$error_type>::$not_found_variant(msg),
          RepositoryError::Conflict(msg) => <$error_type>::$internal_variant(msg),
        }
      }
    }
  };
}
