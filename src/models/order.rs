use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub event_id: ObjectId,
    pub buyer_name: String,
    pub buyer_email: String,
    pub buyer_phone: String,
    pub status: OrderStatus,
    pub midtrans_order_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Paid,
    Sent,
    Failed,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Paid => write!(f, "paid"),
            OrderStatus::Sent => write!(f, "sent"),
            OrderStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderPayload {
    pub event_id: String,
    pub buyer_name: String,
    pub buyer_email: String,
    pub buyer_phone: String,
}
