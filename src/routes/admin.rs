use rocket::serde::Deserialize;
use serde_json::json;
use futures::TryStreamExt;

use rocket::{get, post, routes, State, http::Status, response::status::Custom};
use rocket::serde::json::Json;
use mongodb::{bson::{doc, oid::ObjectId}, Database, Collection, options::FindOptions};

use crate::models::{admin::Admin, event::Event, order::{Order, OrderStatus}};
use crate::utils::{auth::{AdminAuth, verify_password, create_jwt}};
use crate::config::Config;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[get("/me")]
pub fn me(admin: AdminAuth) -> String {
    format!("Admin email: {}", admin.email)
}

#[post("/login", data = "<payload>")]
pub async fn login(
    db: &State<Database>, 
    config: &State<Config>, 
    payload: Json<LoginPayload>
) -> Result<Json<serde_json::Value>, Custom<Json<serde_json::Value>>> {
    let collection: Collection<Admin> = db.collection("admins");
    let admin = collection
        .find_one(doc! {"email": &payload.email}, None)
        .await
        .map_err(|e| {
            eprintln!("=== DATABASE ERROR ===");
            eprintln!("Error: {:?}", e);
            eprintln!("Collection: admins");
            eprintln!("Query: {{\"email\": \"{}\"}}", &payload.email);
            Custom(Status::InternalServerError, Json(json!({"error": "Database query failed"})))
        })?;

    if let Some(admin_doc) = admin {
        if verify_password(&payload.password, &admin_doc.password_hash) {
            let token = create_jwt(&admin_doc.email, &config.jwt_secret)
                .map_err(|_| Custom(Status::InternalServerError, Json(json!({"error": "Token generation failed"}))))?;
            Ok(Json(json!({"token": token})))
        } else {
            Err(Custom(Status::Unauthorized, Json(json!({"error": "Invalid credentials"}))))
        }
    } else {
        Err(Custom(Status::Unauthorized, Json(json!({"error": "Invalid credentials"}))))
    }
}

// --- Event Management ---
#[get("/events")]
pub async fn admin_get_events(
    db: &State<Database>, 
    _admin: AdminAuth
) -> Result<Json<Vec<Event>>, Status> {
    let collection: Collection<Event> = db.collection("events");
    let mut cursor = collection
        .find(doc! {}, None)
        .await
        .map_err(|_| Status::InternalServerError)?;
    
    let mut events = Vec::new();
    while let Some(event) = cursor
        .try_next()  // Sekarang akan bekerja karena import TryStreamExt
        .await
        .map_err(|_| Status::InternalServerError)? {
        events.push(event);
    }
    Ok(Json(events))
}

#[post("/events", data = "<payload>")]
pub async fn create_event(
    db: &State<Database>, 
    _admin: AdminAuth, 
    payload: Json<Event>
) -> Result<Json<Event>, Status> {
    let collection: Collection<Event> = db.collection("events");
    let mut new_event = payload.into_inner();
    new_event.available_tickets = new_event.total_tickets;
    
    let result = collection
        .insert_one(&new_event, None)
        .await
        .map_err(|_| Status::InternalServerError)?;
    
    new_event.id = Some(result.inserted_id.as_object_id().unwrap());
    Ok(Json(new_event))
}

// --- Order Management ---
#[get("/orders")]
pub async fn get_orders(
    db: &State<Database>, 
    _admin: AdminAuth
) -> Result<Json<Vec<Order>>, Status> {
    let collection: Collection<Order> = db.collection("orders");
    let find_options = FindOptions::builder()
        .sort(doc! { "created_at": -1 })
        .build();
    
    let mut cursor = collection
        .find(doc! {}, find_options)
        .await
        .map_err(|_| Status::InternalServerError)?;
    
    let mut orders = Vec::new();
    while let Some(order) = cursor
        .try_next()  // Sekarang akan bekerja karena import TryStreamExt
        .await
        .map_err(|_| Status::InternalServerError)? {
        orders.push(order);
    }
    Ok(Json(orders))
}

#[derive(Deserialize)]
pub struct SendTicketPayload {
    subject: String,
    message: String,
}

#[post("/orders/<id>/send_ticket", data = "<payload>")]
pub async fn send_ticket(
    db: &State<Database>, 
    config: &State<Config>, 
    _admin: AdminAuth, 
    id: &str, 
    payload: Json<SendTicketPayload>
) -> Result<Json<serde_json::Value>, Custom<Json<serde_json::Value>>> {
    let collection: Collection<Order> = db.collection("orders");
    let object_id = ObjectId::parse_str(id)
        .map_err(|_| Custom(Status::BadRequest, Json(json!({"error": "Invalid order ID"}))))?;

    let order = collection
        .find_one(doc! {"_id": object_id}, None)
        .await
        .map_err(|_| Custom(Status::InternalServerError, Json(json!({"error": "Database error"}))))?
        .ok_or_else(|| Custom(Status::NotFound, Json(json!({"error": "Order not found"}))))?;

    if order.status != OrderStatus::Paid {
        return Err(Custom(Status::BadRequest, Json(json!({"error": "Order is not paid"}))));
    }

    let resend_from_email = config.resend_from_email.clone();
    let resend_api_key = config.resend_api_key.clone();

    let email_body = format!(
        "Halo {},\n\n{}\n\nTerima kasih atas pembelian tiket Anda.",
        order.buyer_name,
        payload.message
    );

    let resend_payload = json!({
        "from": resend_from_email,
        "to": [order.buyer_email.clone()],
        "subject": payload.subject.clone(),
        "text": email_body
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", resend_api_key))
        .header("Content-Type", "application/json")
        .json(&resend_payload)
        .send()
        .await
        .map_err(|e| Custom(Status::InternalServerError, Json(json!({
            "error": "Failed to send email",
            "details": e.to_string()
        }))))?;

    if response.status().is_success() {
        collection
            .update_one(
                doc! {"_id": object_id}, 
                doc! {"$set": {"status": "sent"}}, 
                None
            )
            .await
            .map_err(|_| Custom(Status::InternalServerError, Json(json!({"error": "Failed to update order status"}))))?;

        Ok(Json(json!({"message": "Ticket sent successfully"})))
    } else {
        let err_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(Custom(Status::InternalServerError, Json(json!({
            "error": "Resend API error",
            "details": err_text
        }))))
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![me, login, admin_get_events, create_event, get_orders, send_ticket]
}