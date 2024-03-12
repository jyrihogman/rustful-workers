use core::fmt;
use std::error::Error;

use jwt_compact::{
    alg::{Hs256, Hs256Key},
    prelude::*,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use worker::{console_error, Env, Request};

pub type Permissions = Vec<String>;

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomClaims {
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

fn verify_token<T>(
    token_string: &str,
    decoding_key: &str,
    env: &Env,
) -> Result<Claims<T>, AuthError>
where
    T: DeserializeOwned + Clone,
{
    let key = env
        .secret(decoding_key)
        .map_err(|_| AuthError::MissingDecodingKey)?;

    let decoding_key = Hs256Key::new(key.to_string().as_bytes());

    let unverified_token =
        UntrustedToken::new(&token_string).map_err(|_| AuthError::InvalidToken)?;

    let token: Token<T> = Hs256
        .validator(&decoding_key)
        .validate(&unverified_token)
        .map_err(|_| {
            console_error!("Failed to validate token: {}", token_string);
            AuthError::Unauthorized
        })?
        .to_owned();

    Ok(token.claims().to_owned())
}

pub async fn authenticate(req: &Request, env: &Env) -> Result<(), AuthError> {
    let kv = env.kv("tokens").map_err(|_| AuthError::MissingTokens)?;
    let token_string = get_token(req)?;

    let claims = verify_token::<CustomClaims>(&token_string, "DECODING_KEY", env)?;

    let secret = kv.get(&claims.custom.client_id).text().await.map_err(|e| {
        console_error!("Failed to get secret for client: {:?}", e);
        AuthError::Unauthorized
    })?;

    let request_permission = match req.method() {
        worker::Method::Get => "read",
        worker::Method::Post => "post",
        worker::Method::Put => "update",
        worker::Method::Delete => "delete",
        _ => return Err(AuthError::Unauthorized),
    };

    match secret {
        Some(secret_value) => {
            if secret_value != claims.custom.client_secret {
                return Err(AuthError::Unauthorized);
            }

            authorize(request_permission, &claims.custom.permissions)
                .then_some(())
                .ok_or(AuthError::Unauthorized)?;

            Ok(())
        }
        None => Err(AuthError::MissingQStashKey),
    }
}

pub fn authorize(required_permission: &str, permissions: &Permissions) -> bool {
    permissions
        .iter()
        .any(|permission| permission.to_lowercase() == required_permission.to_lowercase())
}
