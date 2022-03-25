use std::sync::Arc;

use actix_web::http::StatusCode;
use common_web::messages::ErrMessage;
use email_address::EmailAddress;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct UserAuthRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

pub struct ValidSyntaxUserAuth {
    pub user_email: Arc<String>,
    pub user_password: Arc<String>,
}

impl UserAuthRequest {
    pub fn validate_syntax(self) -> Result<ValidSyntaxUserAuth, ErrMessage> {
        let email_err = ErrMessage::Generic {
            status: StatusCode::BAD_REQUEST,
            message: "Email is required or malformed",
        };
        let passwd_err = ErrMessage::Generic {
            status: StatusCode::BAD_REQUEST,
            message: "Password is required",
        };
        let user_email = self.email.ok_or(email_err.clone())?;
        if !EmailAddress::is_valid(&user_email) {
            return Err(email_err);
        }
        let user_password = self.password.ok_or(passwd_err)?;
        return Ok(ValidSyntaxUserAuth {
            user_email: Arc::new(user_email),
            user_password: Arc::new(user_password),
        });
    }
}

