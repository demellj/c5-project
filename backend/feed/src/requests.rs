use chrono::{DateTime, Utc};
use serde::Deserialize;

pub static DEFAULT_ITEMS_PER_PAGE: i64 = 20;

#[derive(Deserialize)]
pub struct UpdateFeedItemRequest {
    pub caption: String,
}

#[derive(Deserialize)]
pub struct CreateFeedItemRequest {
    pub caption: Option<String>,
}

#[derive(Deserialize)]
pub struct ItemPageRequest {
    pub before: Option<DateTime<Utc>>,
    pub limit: i64
}
