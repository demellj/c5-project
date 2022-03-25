
use chrono::NaiveDateTime;

use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name="users", primary_key("email"))]
pub struct User  {
    pub id: i32,
    pub email: String,
    pub password_hash: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn short(&self) -> &str {
        self.email.as_str()
    }
}
