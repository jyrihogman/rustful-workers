use core::fmt;
use std::error::Error;

use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use worker::{console_error, Env, Request};

pub type Permissions = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims<'a> {
    client_id: &'a str,
    client_secret: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct Secret {
    client_id: String,
    client_secret: String,
    permissions: Permissions,
}

#[derive(Debug)]
pub enum AuthError {
    MissingAuthHeader,
    Unauthorized,
    InvalidToken,
    MissingDecodingKey,
    MissingTokens,
    MissingQStashKey,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthError::MissingAuthHeader => f.write_str("Missing Authorization Header"),
            AuthError::Unauthorized => f.write_str("Unauthorized"),
            AuthError::InvalidToken => f.write_str("Invalid Token"),
            AuthError::MissingQStashKey => f.write_str("Missing QStash Signing key"),
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
        .map_err(|_| AuthError::MissingAuthHeader)?
        .ok_or(AuthError::MissingAuthHeader)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::MissingTokens);
    }

    let token = auth_header[7..].to_string();

    Ok(token)
}

pub async fn authenticate(req: &Request, env: &Env) -> Result<Permissions, AuthError> {
    let kv = env.kv("tokens").map_err(|_| AuthError::MissingTokens)?;
    let decoding_key = env
        .secret("DECODING_KEY")
        .map_err(|_| AuthError::MissingDecodingKey)?
        .to_string();

    let claims = get_token(req).and_then(|token| {
        HS256Key::from_bytes(decoding_key.as_bytes())
            .verify_token::<Secret>(&token, None)
            .map_err(|e| {
                console_error!("Verifying token failed: {}", e.to_string());
                AuthError::InvalidToken
            })
    })?;

    let client_id = claims.custom.client_id;

    let secret = kv
        .get(&client_id)
        .cache_ttl(6000)
        .text()
        .await
        .map_err(|e| {
            console_error!("Failed to get secret for client {}: {}", client_id, e);
            AuthError::Unauthorized
        })?;

    match secret {
        Some(secret_value) => {
            if secret_value != claims.custom.client_secret {
                console_error!("Invalid secret for client {}", secret_value);
                console_error!("Invalid secret for client {}", claims.custom.client_secret);

                return Err(AuthError::Unauthorized);
            }
            Ok(claims.custom.permissions)
        }
        None => Err(AuthError::MissingQStashKey),
    }
}

pub fn authorize(required_permission: &str, permissions: Permissions) -> Result<(), AuthError> {
    permissions
        .contains(&required_permission.to_string())
        .then_some(())
        .ok_or(AuthError::Unauthorized)
}