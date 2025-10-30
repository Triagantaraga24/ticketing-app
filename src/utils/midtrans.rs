use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

use crate::models::{event::Event, order::Order};

#[derive(Debug, Serialize)]
struct MidtransTransactionDetail {
    order_id: String,
    gross_amount: i64,
}

#[derive(Debug, Serialize)]
struct MidtransItemDetail {
    id: String,
    price: i64,
    quantity: i32,
    name: String,
}

#[derive(Debug, Serialize)]
struct MidtransCustomerDetail {
    first_name: String,
    email: String,
    phone: String,
}

#[derive(Debug, Serialize)]
struct MidtransChargeRequest {
    payment_type: String,
    transaction_details: MidtransTransactionDetail,
    item_details: Vec<MidtransItemDetail>,
    customer_details: MidtransCustomerDetail,
}

#[derive(Debug, Deserialize)]
pub struct MidtransChargeResponse {
    pub token: String,
    pub redirect_url: String,
}

pub async fn create_midtrans_transaction(
    order: &Order,
    event: &Event,
    server_key: &str,
) -> Result<MidtransChargeResponse> {
    let client = Client::new();
    let url = "https://app.sandbox.midtrans.com/snap/v1/transactions";

    let transaction_details = MidtransTransactionDetail {
        order_id: order.midtrans_order_id.clone(),
        gross_amount: (event.price * 1000.0) as i64,
    };

    let item_details = vec![MidtransItemDetail {
        id: event.id.as_ref().unwrap().to_hex(),
        price: (event.price * 1000.0) as i64,
        quantity: 1,
        name: event.name.clone(),
    }];

    let customer_details = MidtransCustomerDetail {
        first_name: order.buyer_name.clone(),
        email: order.buyer_email.clone(),
        phone: order.buyer_phone.clone(),
    };

    let charge_payload = MidtransChargeRequest {
        payment_type: "snap".to_string(),
        transaction_details,
        item_details,
        customer_details,
    };

    // ✅ Perbaiki base64 encoding modern (tanpa deprecated)
    let credentials = format!("{}:", server_key);
    let encoded = general_purpose::STANDARD.encode(credentials);
    let auth_header = format!("Basic {}", encoded);

    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&charge_payload)
        .send()
        .await?;

    if response.status().is_success() {
        let midtrans_res: MidtransChargeResponse = response.json().await?;
        Ok(midtrans_res)
    } else {
        let err_text = response.text().await?;
        Err(anyhow::anyhow!("Midtrans API error: {}", err_text))
    }
}