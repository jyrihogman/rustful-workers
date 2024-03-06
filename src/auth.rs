use core::fmt;
use std::error::Error;

use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use worker::{console_error, Env, Request};

use crate::api::qstash::NotificationMessage;

pub type Permissions = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct QStashClaims {
    iss: String,
    sub: String,
    body: NotificationMessage,
}

#[derive(Serialize, Deserialize)]
struct Secret {
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
        return Err(AuthError::InvalidToken);
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

pub fn authenticate_qstash_request(
    req: &Request,
    env: &Env,
) -> Result<JWTClaims<QStashClaims>, AuthError> {
    let upstash_signature_header = req
        .headers()
        .get("Upstash-Signature")
        .map_err(|_| AuthError::MissingAuthHeader)?
        .ok_or(AuthError::MissingAuthHeader)?;

    let verify_token = |key_binding: String| {
        HS256Key::from_bytes(key_binding.as_bytes())
            .verify_token::<QStashClaims>(&upstash_signature_header, None)
            .map_err(|e| {
                console_error!("Verifying token failed, {}", e);
                AuthError::InvalidToken
            })
    };

    env.secret("QSTASH_CURRENT_SIGNING_KEY")
        .map_err(|e| {
            console_error!("QStash signing key missing, {}", e);
            AuthError::MissingQStashKey
        })
        .and_then(|secret| verify_token(secret.to_string()))
}

pub fn authorize(required_permission: &str, permissions: Permissions) -> Result<(), AuthError> {
    permissions
        .contains(&required_permission.to_string())
        .then_some(())
        .ok_or(AuthError::Unauthorized)
}
