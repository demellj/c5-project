use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::web::{block, Data, Json};

use actix_web::{get, post};

use common_web::database::DBConnPool;
use common_web::guards::IsLoggedIn;
use common_web::messages::{ErrMessage, Message, OkMessage};
use common_web::models::User;
use common_web::router::{RouteBuilder, Router};

use serde_json::json;

use common::{
    config::Config,
    jwt::generate_jwt,
    passwords::{self, compare_with_hashed_password},
};

use log::error;

use common_web::schema::users::dsl::*;
use diesel::dsl::now;
use diesel::prelude::*;

use crate::requests::{UserAuthRequest, ValidSyntaxUserAuth};
use crate::responses::AuthResultResponse;

pub struct AuthRouter;
impl Router for AuthRouter {
    fn build(route_builder: RouteBuilder) -> RouteBuilder {
        route_builder
            .mount(register)
            .mount(login)
            .mount(verification)
    }
}

#[get("/verification")]
async fn verification(_auth: IsLoggedIn) -> Message<serde_json::Value> {
    Ok(OkMessage::Success(json!({
        "auth": true,
        "message": "Authenticated"
    })))
}

#[post("")]
async fn register(
    conn: Data<DBConnPool>,
    config: Data<Config>,
    auth: Json<UserAuthRequest>,
) -> Message<AuthResultResponse> {

    // Check email is valid and password is specified
    let ValidSyntaxUserAuth {
        user_email,
        user_password,
    } = auth.into_inner().validate_syntax()?;

    // Check if user already exists...
    let result = block({
        let conn = conn.get()?;
        let user_email = user_email.clone();
        move || users.find(&*user_email).get_result::<User>(&conn) 
    }).await?;

    if result.is_ok() {
        return Err(ErrMessage::Generic {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: "User may already exist",
        });
    }

    // Hash password
    let hashed_pass = passwords::generate_hashed_password(user_password)
        .await
        .map_err(|e| {
            error!("/register: password hash generation error: {}", e);
            return ErrMessage::InternalServerError;
        })?;

    // Create new user in DB, storing hashed password
    let user = block({
        let conn = conn.get()?;
        move || {
            diesel::insert_into(users)
                .values((
                    email.eq(&*user_email),
                    password_hash.eq(hashed_pass),
                    created_at.eq(now),
                    updated_at.eq(now),
                ))
                .get_result::<User>(&conn)
        }
    })
    .await?
    .map_err(|e| {
        error!("/register: Database error: {}", e);
        return ErrMessage::InternalServerError;
    })?;

    let short = user.short().to_string();

    let jwt = generate_jwt(user, config.into_inner()).await.map_err(|e| {
        error!("/register: jwt generation error: {}", e);
        return ErrMessage::InternalServerError;
    })?;

    Ok(OkMessage::Created(AuthResultResponse {
        auth: None,
        token: Some(jwt),
        user: short,
    }))
}

#[post("/login")]
async fn login(
    conn: Data<DBConnPool>,
    config: Data<Config>,
    auth: Json<UserAuthRequest>,
) -> Message<AuthResultResponse> {
    let conn = conn.get()?;

    let unauth_err = ErrMessage::Generic {
        status: StatusCode::UNAUTHORIZED,
        message: "Unauthorized",
    };

    // Check email is valid and password is specified
    let ValidSyntaxUserAuth {
        user_email,
        user_password,
    } = auth.into_inner().validate_syntax()?;

    // Find user...
    let user = block(move || users.find(&*user_email).get_result::<User>(&conn))
        .await?
        .map_err(|_| unauth_err.clone())?;

    let known_pass = Arc::new(user
        .password_hash
        .as_ref()
        .ok_or(unauth_err.clone())?
        .clone());

    // Verify user password matches
    compare_with_hashed_password(user_password, known_pass)
        .await
        .map_err(|_| unauth_err)?;

    let short = user.short().to_string();

    let jwt = generate_jwt(user, config.into_inner()).await.map_err(|e| {
        error!("/login: jwt generation error: {}", e);
        return ErrMessage::InternalServerError;
    })?;

    Ok(OkMessage::Success(AuthResultResponse {
        auth: Some(true),
        token: Some(jwt),
        user: short,
    }))
}
