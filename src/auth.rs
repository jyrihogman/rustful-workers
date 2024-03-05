use core::fmt;
use std::error::Error;

use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use worker::{Env, Request};

pub type Permissions = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Deserialize)]
struct Secret {
    client_id: String,
    client_secret: String,
    permissions: Permissions,
}

#[derive(Debug)]
pub enum AuthError {
    MissingAuthorization,
    Unauthorized,
    InvalidToken,
    MissingDecodingKey,
    MissingTokens,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthError::MissingAuthorization => f.write_str("Missing Authorization"),
            AuthError::Unauthorized => f.write_str("Unauthorized"),
            AuthError::InvalidToken => f.write_str("Invalid Token"),
            AuthError::MissingDecodingKey => f.write_str("Internal Server Error"),
            AuthError::MissingTokens => f.write_str("Internal Server Error"),
        }
    }
}

impl From<AuthError> for std::string::String {
    fn from(val: AuthError) -> Self {
        val.to_string()
    }
}

impl From<AuthError> for worker::Error {
    fn from(val: AuthError) -> Self {
        worker::Error::from(val.to_string())
    }
}

impl Error for AuthError {}

fn get_token(req: &Request) -> Result<String, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .map_err(|_| AuthError::MissingAuthorization)?
        .ok_or(AuthError::MissingAuthorization)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::MissingAuthorization);
    }

    let token = auth_header[7..].to_string();

    Ok(token)
}

pub async fn authenticate(req: &Request, env: &Env) -> Result<Permissions, AuthError> {
    let decoding_key = env
        .secret("DECODING_KEY")
        .map_err(|_| AuthError::MissingDecodingKey)?
        .to_string();

    let key = HS256Key::from_bytes(decoding_key.as_bytes());

    let claims = get_token(req).and_then(|token| {
        key.verify_token::<Claims>(&token, None)
            .map_err(|_| AuthError::InvalidToken)
    })?;

    let kv = env.kv("TOKENS").map_err(|_| AuthError::MissingTokens)?;

    let secret = kv
        .get(&claims.custom.client_id)
        .json::<Secret>()
        .await
        .map_err(|_| AuthError::InvalidToken)?
        .ok_or(AuthError::InvalidToken)?;

    // Check if the client_secret from the token matches the secret from the KV storage
    if secret.client_secret == claims.custom.client_secret {
        Ok(secret.permissions)
    } else {
        Err(AuthError::InvalidToken)
    }
}

pub fn authorize(required_permission: &str, permissions: Permissions) -> Result<(), AuthError> {
    permissions
        .contains(&required_permission.to_string())
        .then_some(())
        .ok_or(AuthError::Unauthorized)
}
