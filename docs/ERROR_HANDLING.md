# エラーハンドリング仕様

## 概要

本プロジェクトでは、層ごとに明確に役割を分けたエラーハンドリングを実装しています。
各層で適切なエラー型を定義し、`From` trait による自動変換を活用することで、ボイラープレートを削減しています。

## エラー変換の全体フロー

```
┌─────────────────┐
│ Repository層    │  sqlx::Error → RepositoryError
└────────┬────────┘
         │ From trait (マクロで生成)
         ↓
┌─────────────────┐
│ Service層       │  RepositoryError → {Domain}ServiceError
└────────┬────────┘
         │ From trait (手動実装)
         ↓
┌─────────────────┐
│ REST層          │  {Domain}ServiceError → AppError
└─────────────────┘
         │ IntoResponse trait
         ↓
    HTTPレスポンス
```

## 各層のエラー型

### 1. Repository層のエラー

**定義場所**: `src/domains/user/repository.rs`

```rust
pub enum RepositoryError {
  DatabaseError(sqlx::Error),
  NotFound(String),
  Conflict(String),
}
```

**責務**:
- データベース操作で発生するエラーをラップ
- ドメインに依存しない汎用的なエラー型

**変換**:
- `sqlx::Error` → `RepositoryError::DatabaseError` (自動変換)

---

### 2. Service層のエラー

**定義場所**: 
- `src/domains/user/service.rs`
- `src/domains/picture/service.rs`

#### UserServiceError

```rust
pub enum UserServiceError {
  Unauthorized(String),
  ValidationError(String),
  InternalServerError(String),
  InvalidToken(String),
  TokenExpired(String),
  TokenAlreadyUsed(String),
  UserNotFound(String),
}
```

#### PictureServiceError

```rust
pub enum PictureServiceError {
  InternalServerError(String),
  BadRequest(String),
}
```

**責務**:
- ビジネスロジックに関連するエラーを表現
- ドメイン固有のエラーバリアントを持つ

**変換** (マクロで自動生成):
- `sqlx::Error` → `{Domain}ServiceError::InternalServerError`
- `RepositoryError` → `{Domain}ServiceError` (適切なバリアントに変換)

---

### 3. REST層のエラー (AppError)

**定義場所**: `src/utils/error.rs`

```rust
pub struct AppError {
  pub status_code: StatusCode,
  pub message: String,
}
```

**責務**:
- HTTPステータスコードとメッセージを持つ統一エラー型
- `IntoResponse` trait により自動的にJSONレスポンスに変換

**レスポンス形式**:

```json
{
  "error": "エラーメッセージ",
  "status_code": 400
}
```

**変換**:
- `UserServiceError` → `AppError` (手動実装)
- `PictureServiceError` → `AppError` (手動実装)
- その他の一般的なエラー型 (`sqlx::Error`, `serde_json::Error` など) → `AppError`

---

## マクロによる共通化

### `impl_service_error_conversions!` マクロ

**定義場所**: `src/error.rs`

Service層のエラー変換ボイラープレートを削減するためのマクロです。

#### 使用方法

##### パターン1: `sqlx::Error` のみ変換

```rust
impl_service_error_conversions!(PictureServiceError, InternalServerError);
```

**生成されるコード**:
```rust
impl From<sqlx::Error> for PictureServiceError {
  fn from(err: sqlx::Error) -> Self {
    PictureServiceError::InternalServerError(format!("Database error: {}", err))
  }
}
```

##### パターン2: `sqlx::Error` + `RepositoryError` 変換

```rust
impl_service_error_conversions!(UserServiceError, InternalServerError, UserNotFound);
```

**生成されるコード**:
```rust
impl From<sqlx::Error> for UserServiceError {
  fn from(err: sqlx::Error) -> Self {
    UserServiceError::InternalServerError(format!("Database error: {}", err))
  }
}

impl From<RepositoryError> for UserServiceError {
  fn from(err: RepositoryError) -> Self {
    match err {
      RepositoryError::DatabaseError(e) => 
        UserServiceError::InternalServerError(format!("Database error: {}", e)),
      RepositoryError::NotFound(msg) => 
        UserServiceError::UserNotFound(msg),
      RepositoryError::Conflict(msg) => 
        UserServiceError::InternalServerError(msg),
    }
  }
}
```

---

## REST層でのエラーハンドリング

### 基本パターン

Service層のエラーをREST層で自動変換するには、`.map_err(Into::into)` を使用します。

**例**: `src/domains/user/rest.rs`

```rust
pub async fn create_user_handler(
  State(state): State<SharedAppState>,
  Json(payload): Json<CreateUserRequest>,
) -> Result<JsonResponse<User>, AppError> {
  state
    .create_user(payload)
    .await
    .map(JsonResponse)
    .map_err(Into::into)  // UserServiceError → AppError に自動変換
}
```

### エラーマッピングの実装

**定義場所**: `src/utils/error.rs`

各ServiceErrorから`AppError`への変換を定義:

```rust
impl From<UserServiceError> for AppError {
  fn from(error: UserServiceError) -> Self {
    match error {
      UserServiceError::ValidationError(msg) => AppError::bad_request(msg),
      UserServiceError::InternalServerError(msg) => AppError::internal_server_error(msg),
      UserServiceError::Unauthorized(msg) => AppError::unauthorized(msg),
      UserServiceError::InvalidToken(msg) => AppError::bad_request(msg),
      UserServiceError::TokenExpired(msg) => AppError::new(StatusCode::GONE, msg),
      UserServiceError::TokenAlreadyUsed(msg) => AppError::new(StatusCode::CONFLICT, msg),
      UserServiceError::UserNotFound(msg) => AppError::not_found(msg),
    }
  }
}
```

---

## 新しいドメインを追加する際の手順

### 1. Service層エラー型を定義

```rust
// src/domains/new_domain/service.rs
#[derive(Debug)]
pub enum NewDomainServiceError {
  InternalServerError(String),
  BadRequest(String),
  NotFound(String),
}

impl Error for NewDomainServiceError {}

impl std::fmt::Display for NewDomainServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NewDomainServiceError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
      NewDomainServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
      NewDomainServiceError::NotFound(msg) => write!(f, "Not Found: {}", msg),
    }
  }
}
```

### 2. マクロでエラー変換を生成

```rust
use crate::impl_service_error_conversions;

// RepositoryError を使う場合
impl_service_error_conversions!(NewDomainServiceError, InternalServerError, NotFound);

// もしくは sqlx::Error のみの場合
impl_service_error_conversions!(NewDomainServiceError, InternalServerError);
```

### 3. AppError への変換を実装

```rust
// src/utils/error.rs
impl From<crate::domains::new_domain::service::NewDomainServiceError> for AppError {
  fn from(error: crate::domains::new_domain::service::NewDomainServiceError) -> Self {
    use crate::domains::new_domain::service::NewDomainServiceError;
    match error {
      NewDomainServiceError::InternalServerError(msg) => AppError::internal_server_error(msg),
      NewDomainServiceError::BadRequest(msg) => AppError::bad_request(msg),
      NewDomainServiceError::NotFound(msg) => AppError::not_found(msg),
    }
  }
}
```

### 4. REST層でエラーを使用

```rust
// src/domains/new_domain/rest.rs
pub async fn handler(
  State(state): State<SharedAppState>,
) -> Result<JsonResponse<Response>, AppError> {
  state
    .do_something()
    .await
    .map(JsonResponse)
    .map_err(Into::into)  // 自動変換
}
```

---

## ベストプラクティス

### ✅ 推奨

1. **REST層では `.map_err(Into::into)` を使う**
   - `From` trait による自動変換を活用
   - ボイラープレートを削減

2. **Service層のエラーはマクロで生成**
   - `impl_service_error_conversions!` を使用
   - 重複コードを排除

3. **ドメイン固有のエラーバリアントを定義**
   - ビジネスロジックに関連するエラーはServiceError層で表現
   - HTTPステータスへの変換はAppError層で行う

### ❌ 避けるべき

1. **REST層で個別のエラー変換関数を定義しない**
   ```rust
   // ❌ 避ける
   fn map_service_error(e: ServiceError) -> AppError { ... }
   
   // ✅ 推奨
   .map_err(Into::into)
   ```

2. **Service層で直接HTTPステータスコードを扱わない**
   - Service層はHTTPに依存しない
   - ステータスコードへの変換はAppError層で行う

3. **エラーメッセージに機密情報を含めない**
   - スタックトレースや内部実装の詳細をクライアントに返さない
   - ログには詳細情報を出力し、クライアントには汎用的なメッセージを返す

---

## トラブルシューティング

### マクロが見つからないエラー

```
error: cannot find macro `impl_service_error_conversions` in this scope
```

**解決方法**:
```rust
use crate::impl_service_error_conversions;
```
をファイルの冒頭に追加してください。

### From trait の実装が見つからないエラー

```
error[E0277]: the trait `From<DomainServiceError>` is not implemented for `AppError`
```

**解決方法**:
`src/utils/error.rs` に `impl From<DomainServiceError> for AppError` を追加してください。

---

## 参考

- **エラーマクロ定義**: `src/error.rs`
- **AppError定義**: `src/utils/error.rs`
- **使用例**:
  - User domain: `src/domains/user/service.rs`, `src/domains/user/rest.rs`
  - Picture domain: `src/domains/picture/service.rs`, `src/domains/picture/rest.rs`
