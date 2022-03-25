use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::web::{block, Data, Json, Path, Query};

use actix_web::{delete, get, patch, post};

use common::config::Config;
use uuid::Uuid;

use chrono::Utc;
use common::aws::s3::Media;
use common::aws::S3Bucket;

use common_web::database::DBConnPool;
use common_web::guards::IsLoggedIn;
use common_web::messages::{ErrMessage, Message, OkMessage};
use common_web::models::FeedItem;
use common_web::router::{RouteBuilder, Router};

use common_web::schema::feeditems::dsl::*;
use diesel::prelude::*;

use crate::requests::{
    CreateFeedItemRequest, ItemPageRequest, UpdateFeedItemRequest, DEFAULT_ITEMS_PER_PAGE,
};
use crate::responses::FeedItemResponse;

use log::error;

pub struct FeedRouter;
impl Router for FeedRouter {
    fn build(route_builder: RouteBuilder) -> RouteBuilder {
        route_builder
            .mount(get_all_feeds)
            .mount(get_all_thumbnails)
            .mount(get_feed)
            .mount(get_feed_thumbnail)
            .mount(update_feed)
            .mount(create_feed)
            .mount(delete_feed)
            .mount(get_signed_url)
    }
}

#[get("")]
async fn get_all_feeds(
    auth: Option<IsLoggedIn>,
    conn: Data<DBConnPool>,
    media_bucket: Data<S3Bucket<Media>>,
    query: Option<Query<ItemPageRequest>>,
) -> Message<Vec<FeedItemResponse>> {
    let user = auth.map(IsLoggedIn::get_user);

    let conn = conn.get()?;

    // Get UTC timestamp
    let (limit, timestamp) = if let Some(query) = query {
        let ItemPageRequest { before, limit } = query.into_inner();
        // If no timestamp specified, use current time in UTC
        let before = before.unwrap_or(Utc::now());
        (limit, before.naive_utc())
    } else {
        // If none specified, use current time in UTC
        (DEFAULT_ITEMS_PER_PAGE, Utc::now().naive_utc())
    };

    // Return items older than provided timestamp
    let feed_items = block(move || {
        feeditems
            .filter(updated_at.lt(timestamp))
            .order_by(updated_at.desc())
            .limit(limit)
            .load::<FeedItem>(&conn)
    })
    .await??;

    let mut returned_feeds = Vec::<FeedItemResponse>::new();
    for feed_item in feed_items {
        let result = media_bucket
            .get_object_presigned_url(&feed_item.image_id)
            .await;
        match result {
            Ok(presigned_url) => {
                if let Some(ref user) = user {
                    // If the user is logged in we update the editable field in the response to
                    // true only if the user was also the creator of the feed item.
                    returned_feeds.push((user, presigned_url, feed_item).into());
                } else {
                    // Otherwise the feed item is not editable
                    returned_feeds.push((presigned_url, feed_item).into());
                }
            }
            Err(err) => error!("s3: {}", err),
        }
    }

    Ok(OkMessage::Success(returned_feeds))
}

#[get("/thumbnails")]
async fn get_all_thumbnails(
    conn: Data<DBConnPool>,
    config: Data<Config>,
    query: Option<Query<ItemPageRequest>>,
) -> Message<Vec<FeedItemResponse>> {
    let conn = conn.get()?;

    // Get UTC timestamp
    let (limit, timestamp) = if let Some(query) = query {
        let ItemPageRequest { before, limit } = query.into_inner();
        // If no timestamp specified, use current time in UTC
        let before = before.unwrap_or(Utc::now());
        (limit, before.naive_utc())
    } else {
        // If none specified, use current time in UTC
        (DEFAULT_ITEMS_PER_PAGE, Utc::now().naive_utc())
    };

    // Return items older than provided timestamp
    let feed_items = block(move || {
        feeditems
            .filter(updated_at.lt(timestamp))
            .order_by(updated_at.desc())
            .limit(limit)
            .load::<FeedItem>(&conn)
    })
    .await??;

    let mut returned_feeds = Vec::<FeedItemResponse>::new();
    for feed_item in feed_items {
        let url = format!(
            "{}/{}",
            config.aws_thumbnails_base_url,
            feed_item.image_id
        );

        returned_feeds.push((url, feed_item).into());
    }

    Ok(OkMessage::Success(returned_feeds))
}

#[get("/{feed_id}")]
async fn get_feed(
    auth: IsLoggedIn,
    conn: Data<DBConnPool>,
    media_bucket: Data<S3Bucket<Media>>,
    feed_id: Path<i32>,
) -> Message<FeedItemResponse> {
    let user = auth.get_user();

    let conn = conn.get()?;

    let feed_id = feed_id.into_inner();

    let feed_item = block(move || feeditems.find(feed_id).get_result::<FeedItem>(&conn))
        .await?
        .map_err(|err| {
            error!("diesel: {}", err);
            ErrMessage::Generic {
                status: StatusCode::NOT_FOUND,
                message: "Feed item not found",
            }
        })?;

    let result = media_bucket
        .get_object_presigned_url(&feed_item.image_id)
        .await;
    match result {
        Ok(presigned_url) => {
            return Ok(OkMessage::Success((&user, presigned_url, feed_item).into()));
        }
        Err(err) => error!("s3: {}", err),
    }

    Err(ErrMessage::InternalServerError)
}

#[get("/{feed_id}/thumbnail")]
async fn get_feed_thumbnail(
    conn: Data<DBConnPool>,
    config: Data<Config>,
    feed_id: Path<i32>,
) -> Message<FeedItemResponse> {
    let conn = conn.get()?;

    let feed_id = feed_id.into_inner();

    let feed_item = block(move || feeditems.find(feed_id).get_result::<FeedItem>(&conn))
        .await?
        .map_err(|err| {
            error!("diesel: {}", err);
            ErrMessage::Generic {
                status: StatusCode::NOT_FOUND,
                message: "Feed item not found",
            }
        })?;

    let url = format!(
        "{}/{}",
        config.aws_thumbnails_base_url,
        feed_item.image_id
    );

    Ok(OkMessage::Success((url, feed_item).into()))
}

#[patch("/{feed_id}")]
async fn update_feed(
    auth: IsLoggedIn,
    conn: Data<DBConnPool>,
    media_bucket: Data<S3Bucket<Media>>,
    feed_id: Path<i32>,
    feed: Json<UpdateFeedItemRequest>,
) -> Message<FeedItemResponse> {
    let conn = conn.get()?;

    let Json(feed) = feed;

    let feed_id = feed_id.into_inner();

    let user = Arc::new(auth.get_user());

    let feed_item = block({
        let user = user.clone();
        move || {
            diesel::update(feeditems)
                .filter(id.eq(feed_id))
                .filter(created_by.eq(&*user.email))
                .set((caption.eq(feed.caption), updated_at.eq(diesel::dsl::now)))
                .get_result::<FeedItem>(&conn)
        }
    })
    .await?
    .map_err(|err| {
        error!("diesel: {}", err);
        ErrMessage::Generic {
            status: StatusCode::BAD_REQUEST,
            message: "Feed item not found or item not editable by user",
        }
    })?;

    let result = media_bucket
        .get_object_presigned_url(&feed_item.image_id)
        .await;
    match result {
        Ok(presigned_url) => {
            return Ok(OkMessage::Success(
                (&*user, presigned_url, feed_item).into(),
            ));
        }
        Err(err) => error!("s3: {}", err),
    }

    Err(ErrMessage::InternalServerError)
}

#[post("")]
async fn create_feed(
    auth: IsLoggedIn,
    conn: Data<DBConnPool>,
    feed: Json<CreateFeedItemRequest>,
    media_bucket: Data<S3Bucket<Media>>,
) -> Message<FeedItemResponse> {
    let conn = conn.get()?;

    let user = Arc::new(auth.get_user());

    let Json(feed) = feed;

    if feed.caption.is_none() {
        return Err(ErrMessage::Generic {
            status: StatusCode::BAD_REQUEST,
            message: "Caption is required or malformed",
        });
    }

    let feed_image_id = Uuid::new_v4().to_simple().to_string();

    let feed_item = block({
        let user = user.clone();
        move || {
            diesel::insert_into(feeditems)
                .values(&vec![(
                    caption.eq(feed.caption),
                    image_id.eq(feed_image_id),
                    created_by.eq(&*user.email),
                    created_at.eq(diesel::dsl::now),
                    updated_at.eq(diesel::dsl::now),
                )])
                .get_result::<FeedItem>(&conn)
        }
    })
    .await??;

    let feed_url = media_bucket
        .put_object_presigned_url(&feed_item.image_id)
        .await
        .map_err(|err| {
            error!("s3: {}", err);
            ErrMessage::InternalServerError
        })?;

    Ok(OkMessage::Created((&*user, feed_url, feed_item).into()))
}

#[delete("/{feed_id}")]
async fn delete_feed(
    auth: IsLoggedIn,
    conn: Data<DBConnPool>,
    media_bucket: Data<S3Bucket<Media>>,
    feed_id: Path<i32>,
) -> Message<serde_json::Value> {
    let conn = conn.get()?;

    let feed_id = feed_id.into_inner();

    let user = auth.get_user();

    let feed_item = block(move || {
        diesel::dsl::delete(feeditems)
            .filter(id.eq(feed_id))
            .filter(created_by.eq(&user.email))
            .get_result::<FeedItem>(&conn)
    })
    .await?
    .map_err(|err| {
        error!("diesel: {}", err);
        ErrMessage::Generic {
            status: StatusCode::BAD_REQUEST,
            message: "Feed item not found or item not editable by user",
        }
    })?;

    media_bucket
        .delete_object(&feed_item.image_id)
        .await
        .map_err(|err| {
            error!("s3: {}", err);
            ErrMessage::InternalServerError
        })?;

    Ok(OkMessage::Success(serde_json::json!({
        "id": feed_item.id
    })))
}

#[get("/signed-url/{file_name}")]
async fn get_signed_url(
    _auth: IsLoggedIn,
    file_name: Path<String>,
    media_bucket: Data<S3Bucket<Media>>,
) -> Message<serde_json::Value> {
    let file_name = file_name.into_inner();

    let presigned_url = media_bucket
        .put_object_presigned_url(&file_name)
        .await
        .map_err(|err| {
            error!("s3: {}", err);
            ErrMessage::InternalServerError
        })?;

    Ok(OkMessage::Success(serde_json::json!({
        "url": presigned_url
    })))
}
