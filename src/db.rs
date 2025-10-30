use mongodb::{Client, options::ClientOptions, Database};
use crate::config::Config;

pub async fn init_db(config: &Config) -> Database {
    let mut client_options = ClientOptions::parse(&config.mongodb_uri).await.unwrap();
    client_options.app_name = Some("ticketing_app".to_string());

    let client = Client::with_options(client_options).unwrap();
    client.database(&config.mongodb_uri.split('/').last().unwrap_or("ticketing_db"))
}