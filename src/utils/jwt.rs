use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: String,
  pub exp: usize,
  pub user_id: i32,
}

pub fn decode_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
  let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());

  let token_data = decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_ref()),
    &Validation::default(),
  )?;

  Ok(token_data.claims)
}

pub fn encode_jwt(claims: Claims) -> Result<String, jsonwebtoken::errors::Error> {
  let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string());

  encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}
