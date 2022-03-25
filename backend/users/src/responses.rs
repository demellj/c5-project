use serde::Serialize;

use chrono::NaiveDateTime;

#[derive(Serialize)]
pub struct UserResponse {
    pub email: String,
    pub created_at: NaiveDateTime, 
}

#[derive(Serialize)]
pub struct AuthResultResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub user: String,
}

