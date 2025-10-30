use dotenvy::dotenv;
use mongodb::{Client, options::ClientOptions, Collection};
use bson::doc;
use chrono::Utc;
use ticketing_app::{
    config::Config,
    models::{admin::Admin, event::Event},
    utils::auth::hash_password,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    dotenv().ok();
    let config = Config::from_env();

    let mut client_options = ClientOptions::parse(&config.mongodb_uri).await?;
    client_options.app_name = Some("ticketing_app_seed".to_string());
    let client = Client::with_options(client_options)?;
    let db_name = config.mongodb_uri.split('/').last().unwrap_or("ticketing_db");
    let db = client.database(db_name);

    // Seed Admin
    let admin_collection: Collection<Admin> = db.collection("admins");
    if admin_collection.count_documents(Some(doc! {}), None).await? == 0 {
        let password_hash = hash_password(&config.admin_password).expect("Failed to hash password");
        let admin = Admin {
            id: None,
            email: config.admin_email.clone(),
            password_hash,
            created_at: Utc::now(),
        };
        admin_collection.insert_one(admin, None).await?;
        println!("✅ Admin user created:");
        println!("   Email: {}", config.admin_email);
        println!("   Password: {}", config.admin_password);
    } else {
        println!("ℹ️ Admin user already exists. Skipping creation.");
    }

    // Seed Events
    let event_collection: Collection<Event> = db.collection("events");
    event_collection.delete_many(doc! {}, None).await?;

    let events = vec![
        Event {
            id: None,
            name: "Jakarta Music Fest".to_string(),
            description: "A night of stellar music performances.".to_string(),
            date: Utc::now() + chrono::Duration::days(30),
            location: "Jakarta Convention Center".to_string(),
            price: 150000.0,
            total_tickets: 500,
            available_tickets: 500,
        },
        Event {
            id: None,
            name: "Comedy Night".to_string(),
            description: "Get ready to laugh out loud with top comedians.".to_string(),
            date: Utc::now() + chrono::Duration::days(15),
            location: "Isola Bar, Jakarta".to_string(),
            price: 75000.0,
            total_tickets: 200,
            available_tickets: 200,
        },
    ];

    event_collection.insert_many(events, None).await?;
    println!("🎟️ Sample events added:");
    println!("   - Jakarta Music Fest");
    println!("   - Comedy Night");
    
    println!("\n🎉 Seeding complete!");
    Ok(())
}