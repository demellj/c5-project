use std::{fmt::Display, sync::Arc};

use argon2::{
    password_hash::{
        rand_core::OsRng, Error as PasswordHashError, PasswordHash, PasswordHasher,
        PasswordVerifier, SaltString,
    },
    Argon2,
};

use tokio::task::{spawn_blocking, JoinError};

#[derive(Debug)]
pub enum PasswordError {
    HashError(PasswordHashError),
    AsyncError(JoinError),
}

impl Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::HashError(e) => write!(f, "{}", e),
            PasswordError::AsyncError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PasswordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PasswordError::HashError(_) => None,
            PasswordError::AsyncError(e) => Some(e),
        }
    }
}

impl From<PasswordHashError> for PasswordError {
    fn from(e: PasswordHashError) -> Self {
        PasswordError::HashError(e)
    }
}

impl From<JoinError> for PasswordError {
    fn from(e: JoinError) -> Self {
        PasswordError::AsyncError(e)
    }
}

pub async fn generate_hashed_password(password: Arc<String>) -> Result<String, PasswordError> {
    let hash = spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    })
    .await??;

    Ok(hash)
}

pub async fn compare_with_hashed_password(
    plain_text_password: Arc<String>,
    hashed_password: Arc<String>,
) -> Result<(), PasswordError> {
    spawn_blocking(move || {
        PasswordHash::new(&hashed_password).map(|parsed_hash| {
            Argon2::default().verify_password(plain_text_password.as_bytes(), &parsed_hash)
        })
    })
    .await???;

    Ok(())
}
