use std::error::Error;
use std::fmt::Display;
use std::sync::PoisonError;

use actix_web::body::BoxBody;

use serde::Serialize;
use serde_json::json;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder, ResponseError};

use log::error;

pub type Message<T> = Result<OkMessage<T>, ErrMessage>;

pub enum OkMessage<T: Serialize> {
    Success(T),
    Created(T),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ErrMessage {
    Generic {
        status: StatusCode,
        message: &'static str,
    },
    InternalServerError,
}

impl Display for ErrMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrMessage::Generic { status, message } => {
                write!(f, "Status: {}, Message: {}", status, message)
            }
            ErrMessage::InternalServerError => {
                write!(f, "Status: 500, Message: Internal Server Error")
            }
        }
    }
}

impl From<actix_web::error::BlockingError> for ErrMessage {
    fn from(err: actix_web::error::BlockingError) -> Self {
        error!("blocking: {}", err);
        ErrMessage::InternalServerError
    }
}

impl From<diesel::result::Error> for ErrMessage {
    fn from(err: diesel::result::Error) -> Self {
        error!("diesel: {}", err);
        ErrMessage::InternalServerError
    }
}

impl From<r2d2::Error> for ErrMessage {
    fn from(err: r2d2::Error) -> Self {
        error!("r2d2: {}", err);
        ErrMessage::InternalServerError
    }
}

impl<T> From<PoisonError<T>> for ErrMessage {
    fn from(err: PoisonError<T>) -> Self {
        error!("mutext poisoning: {}", err);
        ErrMessage::InternalServerError
    }
}

impl Error for ErrMessage {}

impl ResponseError for ErrMessage {
    fn status_code(&self) -> StatusCode {
        match self {
            ErrMessage::Generic { status, .. } => status.clone(),
            ErrMessage::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ErrMessage::Generic { status, message } => {
                error!("Generic Error: status:{}, message:{}", status, message);
                HttpResponse::build(self.status_code()).json(json!({
                  "code": status.as_u16(),
                  "message": message
                }))
            }
            ErrMessage::InternalServerError => internal_server_error(),
        }
    }
}

impl<T: Serialize> Responder for OkMessage<T> {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            Self::Success(msg) => HttpResponse::Ok().json(msg),
            Self::Created(msg) => HttpResponse::Created().json(msg),
        }
    }
}

fn internal_server_error() -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        "message": "An internal error has occurred"
    }))
}
