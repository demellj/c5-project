use common_web::{
    database::DBConnPool,
    messages::{Message, OkMessage},
    router::{RouteBuilder, Router}, models::User,
};

use actix_web::{
    get,
    web::{block, Data, Path},
};

use common_web::schema::users::dsl::*;
use diesel::prelude::*;

mod auth;
use auth::AuthRouter;

use crate::responses::UserResponse;

pub struct UserRouter;
impl Router for UserRouter {
    fn build(route_builder: RouteBuilder) -> RouteBuilder {
        route_builder
            .mount(get_user_by_id)
            .extend::<AuthRouter>("auth")
    }
}

#[get("/{user_email}")]
async fn get_user_by_id(conn: Data<DBConnPool>, user_email: Path<String>) -> Message<UserResponse> {
    let conn = conn.get()?;

    let user_email = user_email.into_inner();

    let user = block(move || users.find(user_email).get_result::<User>(&conn)).await??;

    Ok(OkMessage::Success(UserResponse{
        email: user.email,
        created_at: user.created_at
    }))
}
