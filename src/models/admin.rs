use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Admin {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    #[serde(default = "chrono::Utc::now")]
    pub created_at: DateTime<Utc>,
}