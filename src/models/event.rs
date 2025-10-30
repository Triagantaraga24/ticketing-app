use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub date: DateTime<Utc>,
    pub location: String,
    pub price: f64,
    pub total_tickets: i32,
    pub available_tickets: i32,
}