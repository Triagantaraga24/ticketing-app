use rocket::{get, post, routes, State, http::Status, response::status::Custom};
use rocket::serde::json::Json;
use mongodb::{bson::{doc, oid::ObjectId}, Database, Collection};
use uuid::Uuid;
use serde_json::json;
use futures::TryStreamExt;
use anyhow::Result;

use crate::models::{event::Event, order::{Order, CreateOrderPayload, OrderStatus}};
use crate::utils::midtrans::create_midtrans_transaction;
use crate::config::Config;

#[get("/events")]
pub async fn get_events(db: &State<Database>) -> Result<Json<Vec<Event>>, Status> {
    let collection: Collection<Event> = db.collection("events");
    let mut cursor = collection.find(doc! {}, None).await.map_err(|_| Status::InternalServerError)?;
    let mut events = Vec::new();
    while let Some(event) = cursor.try_next().await.map_err(|_| Status::InternalServerError)? {
        events.push(event);
    }
    Ok(Json(events))
}

#[get("/events/<id>")]
pub async fn get_event(db: &State<Database>, id: &str) -> Result<Json<Event>, Status> {
    let collection: Collection<Event> = db.collection("events");
    let object_id = ObjectId::parse_str(id).map_err(|_| Status::BadRequest)?;
    let event = collection.find_one(doc! {"_id": object_id}, None).await.map_err(|_| Status::InternalServerError)?;
    
    match event {
        Some(e) => Ok(Json(e)),
        None => Err(Status::NotFound),
    }
}

#[post("/orders", data = "<payload>")]
pub async fn create_order(
    db: &State<Database>, 
    config: &State<Config>, 
    payload: Json<CreateOrderPayload>
) -> Result<Json<serde_json::Value>, Custom<Json<serde_json::Value>>> {
    let event_collection: Collection<Event> = db.collection("events");
    let order_collection: Collection<Order> = db.collection("orders");

    let event_object_id = ObjectId::parse_str(&payload.event_id)
        .map_err(|_| Custom(Status::BadRequest, Json(json!({"error": "Invalid event ID"}))))?;
    
    let event = event_collection.find_one(doc! {"_id": event_object_id}, None)
        .await
        .map_err(|_| Custom(Status::InternalServerError, Json(json!({"error": "Database error"}))))?;
    
    let event = event.ok_or_else(|| Custom(Status::NotFound, Json(json!({"error": "Event not found"}))))?;

    if event.available_tickets <= 0 {
        return Err(Custom(Status::BadRequest, Json(json!({"error": "No tickets available"}))));
    }

    let midtrans_order_id = format!("ORDER-{}", Uuid::new_v4());
    let new_order = Order {
        id: None,
        event_id: event_object_id,
        buyer_name: payload.buyer_name.clone(),
        buyer_email: payload.buyer_email.clone(),
        buyer_phone: payload.buyer_phone.clone(),
        status: OrderStatus::Pending,
        midtrans_order_id: midtrans_order_id.clone(),
        created_at: chrono::Utc::now(),
    };

    // Tangkap error dengan lebih baik
    let insert_result = match order_collection.insert_one(&new_order, None).await {
        Ok(result) => result,
        Err(_) => return Err(Custom(Status::InternalServerError, Json(json!({"error": "Failed to create order"})))),
    };
    
    match create_midtrans_transaction(&new_order, &event, &config.midtrans_server_key).await {
        Ok(midtrans_res) => {
            Ok(Json(json!({
                "order_id": insert_result.inserted_id.as_object_id().unwrap().to_hex(),
                "midtrans_token": midtrans_res.token,
                "redirect_url": midtrans_res.redirect_url
            })))
        }
        Err(e) => {
            // Cleanup failed order - abaikan error cleanup
            let _ = order_collection.delete_one(doc! {"_id": insert_result.inserted_id}, None).await;
            
            // Gunakan error message yang aman
            Err(Custom(
                Status::InternalServerError, 
                Json(json!({
                    "error": "Failed to create payment transaction",
                    "details": format!("Payment service error: {}", e)
                }))
            ))
        }
    }
}

#[post("/orders/notify", data = "<payload>")]
pub async fn midtrans_webhook(
    db: &State<Database>, 
    _config: &State<Config>, 
    payload: Json<serde_json::Value>
) -> Status {
    println!("=== MIDTRANS WEBHOOK RECEIVED ===");
    println!("Payload: {:?}", payload);
    
    if let Some(order_id) = payload.get("order_id").and_then(|v| v.as_str()) {
        println!("Processing order_id: {}", order_id);
        
        if let Some(transaction_status) = payload.get("transaction_status").and_then(|v| v.as_str()) {
            println!("Transaction status: {}", transaction_status);
            
            let collection: Collection<Order> = db.collection("orders");
            
            // Tentukan status baru
            let new_status = if transaction_status == "settlement" {
                OrderStatus::Paid
            } else if transaction_status == "deny" || transaction_status == "cancel" || transaction_status == "expire" {
                OrderStatus::Failed
            } else {
                println!("Transaction status ignored: {}", transaction_status);
                return Status::Ok;
            };

            println!("New status to set: {:?}", new_status);
            
            // ✅ Buat update document
            let update = doc! { "$set": { "status": new_status.to_string() } };
            
            // ✅ Coba update dengan _id (ObjectId)
            if let Ok(object_id) = ObjectId::parse_str(order_id) {
                let filter = doc! { "_id": object_id };
                
                match collection.update_one(filter, update.clone(), None).await {
                    Ok(result) => {
                        println!("Update by _id - matched: {}, modified: {}", 
                                 result.matched_count, result.modified_count);
                        if result.matched_count > 0 {
                            println!("✅ Order updated successfully by _id");
                            return Status::Ok;
                        }
                    }
                    Err(e) => {
                        println!("❌ Database update error by _id: {:?}", e);
                    }
                }
            }
            
            // ✅ Jika update pertama gagal, coba dengan field lain
            let filter = doc! { "midtrans_order_id": order_id };
            match collection.update_one(filter, update, None).await {
                Ok(result) => {
                    println!("Update by midtrans_order_id - matched: {}, modified: {}", 
                             result.matched_count, result.modified_count);
                }
                Err(e) => {
                    println!("❌ Database update error by midtrans_order_id: {:?}", e);
                }
            }
        }
    }
    
    Status::Ok
}

pub fn routes() -> Vec<rocket::Route> {
    routes![get_events, get_event, create_order, midtrans_webhook]
}