use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    pub mongodb_uri: String,
    pub jwt_secret: String,
    pub admin_email: String,
    pub admin_password: String,
    pub midtrans_server_key: String,
    pub midtrans_client_key: String,
    pub resend_api_key: String,
    pub resend_from_email: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            mongodb_uri: env::var("MONGODB_URI").expect("MONGODB_URI must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            admin_email: env::var("ADMIN_EMAIL").expect("ADMIN_EMAIL must be set"),
            admin_password: env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set"),
            midtrans_server_key: env::var("MIDTRANS_SERVER_KEY").expect("MIDTRANS_SERVER_KEY must be set"),
            midtrans_client_key: env::var("MIDTRANS_CLIENT_KEY").expect("MIDTRANS_CLIENT_KEY must be set"),
            resend_api_key: env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set"),
            resend_from_email: env::var("RESEND_FROM_EMAIL").expect("RESEND_FROM_EMAIL must be set"),
        }
    }
}