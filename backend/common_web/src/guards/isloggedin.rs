use std::future::Future;
use std::pin::Pin;

use actix_web::FromRequest;
use actix_web::{http::StatusCode, web::Data};
use common::{config::Config, jwt::verify_jwt};

use crate::{messages::ErrMessage, models::User};

use log::debug;

// Define an extractor to check for logged in users
// IMPORTANT: ensure this it NOT constructable outside this module
pub struct IsLoggedIn(User);

impl IsLoggedIn {
    pub fn get_user(self) -> User {
        self.0
    }
}

impl FromRequest for IsLoggedIn {
    type Error = ErrMessage;

    type Future = Pin<Box<dyn Future<Output = Result<IsLoggedIn, ErrMessage>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let auth = req
                .headers()
                .get("Authorization")
                .ok_or(ErrMessage::Generic {
                    status: StatusCode::UNAUTHORIZED,
                    message: "No authorization headers",
                })?;

            let config = req
                .app_data::<Data<Config>>()
                .ok_or(ErrMessage::InternalServerError)?;

            let auth = auth.to_str().map_err(|_| ErrMessage::Generic {
                status: StatusCode::UNAUTHORIZED,
                message: "Malformed token",
            })?;

            // input of the form: Bearer xxxxxxxx
            let (_, token) = auth.split_once(" ").ok_or(ErrMessage::Generic {
                status: StatusCode::UNAUTHORIZED,
                message: "Malformed token",
            })?;

            let user = verify_jwt::<User>(token.to_string(), config.clone().into_inner())
                .await
                .map_err(|e| { 
                    debug!("jwt_verify: {} token: {}", e, token);
                    ErrMessage::Generic {
                        status: StatusCode::UNAUTHORIZED,
                        message: "Failed to authenticate",
                    }
                })?;

            Ok(IsLoggedIn(user))
        })
    }
}
