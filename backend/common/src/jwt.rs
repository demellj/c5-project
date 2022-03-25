use std::{
    fmt::Display,
    ops::Add,
    sync::Arc,
    time::{SystemTime, SystemTimeError},
};

use jsonwebtoken as jwt;
use tokio::task::JoinError;
use serde::{de, Deserialize, Serialize};

use crate::config::Config;

pub const DEFAULT_TIMEOUT_IN_SEC: u64 = 60 * 60;

#[derive(Serialize, Deserialize)]
struct JWTClaims<T>
where
    T: Serialize + de::DeserializeOwned,
{
    #[serde(bound(deserialize = "T: serde::de::DeserializeOwned"))]
    data: T,
    exp: usize, // this field is vaildated
}

#[derive(Debug)]
pub enum JWTError {
    JWTError(jwt::errors::Error),
    AsyncError(JoinError),
    SystemTimeError(SystemTimeError),
}

impl Display for JWTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JWTError::JWTError(e) => write!(f, "{}", e),
            JWTError::AsyncError(e) => write!(f, "{}", e),
            JWTError::SystemTimeError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for JWTError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JWTError::JWTError(_) => None,
            JWTError::AsyncError(e) => Some(e),
            JWTError::SystemTimeError(e) => Some(e),
        }
    }
}

impl From<jwt::errors::Error> for JWTError {
    fn from(e: jwt::errors::Error) -> Self {
        JWTError::JWTError(e)
    }
}

impl From<JoinError> for JWTError {
    fn from(e: JoinError) -> Self {
        JWTError::AsyncError(e)
    }
}

impl From<SystemTimeError> for JWTError {
    fn from(e: SystemTimeError) -> Self {
        JWTError::SystemTimeError(e)
    }
}

pub async fn generate_jwt<T>(data: T, config: Arc<Config>) -> Result<String, JWTError>
where
    T: Serialize + de::DeserializeOwned + Send + 'static,
{
    let result =
        tokio::task::spawn_blocking(move || generate_jwt_helper(data, config)).await??;

    Ok(result)
}

fn generate_jwt_helper<T>(data: T, config: Arc<Config>) -> Result<String, JWTError>
where
    T: Serialize + de::DeserializeOwned + Send + 'static,
{
    let exp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .add(config.jwt_token_timeout)
        .as_secs();

    let exp = exp as usize;

    let claim = JWTClaims { data, exp };

    let result = jwt::encode(
        &jwt::Header::default(),
        &claim,
        &jwt::EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(result)
}

pub async fn verify_jwt<T>(token: String, config: Arc<Config>) -> Result<T, JWTError>
where
    T: Serialize + de::DeserializeOwned + Send + 'static,
{
    let result = tokio::task::spawn_blocking(move || {
        jwt::decode::<JWTClaims<T>>(
            token.as_str(),
            &jwt::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &jwt::Validation::default(),
        )
        .map(|v| v.claims.data)
    })
    .await??;

    Ok(result)
}
