use serde::{Serialize, Deserialize};

use chrono::NaiveDateTime;

#[derive(Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name="feeditems")]
pub struct FeedItem {
    pub id: i32,
    pub created_by: String,
    pub image_id: String,
    pub caption: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

