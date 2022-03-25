use common_web::models::{FeedItem, User};
use serde::Serialize;

use chrono::{DateTime, Utc};

#[derive(Serialize, Debug)]
pub struct FeedItemResponse {
    pub id: i32,
    pub caption: Option<String>,
    pub url: String,
    pub editable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<(String, FeedItem)> for FeedItemResponse {
    fn from((url, item): (String, FeedItem)) -> Self {
        let FeedItem {
            id,
            caption,
            created_at,
            updated_at,
            ..
        } = item;

        FeedItemResponse {
            id,
            caption,
            url,
            editable: false,
            created_at: DateTime::<Utc>::from_utc(created_at, Utc),
            updated_at: DateTime::<Utc>::from_utc(updated_at, Utc),
        }
    }
}

impl<'a> From<(&'a User, String, FeedItem)> for FeedItemResponse {
    fn from((user, url, item): (&'a User, String, FeedItem)) -> Self {
        let FeedItem {
            id,
            caption,
            created_at,
            updated_at,
            created_by,
            ..
        } = item;

        FeedItemResponse {
            id,
            caption,
            url,
            editable: user.email.eq(&created_by),
            created_at: DateTime::<Utc>::from_utc(created_at, Utc),
            updated_at: DateTime::<Utc>::from_utc(updated_at, Utc),
        }
    }
}
