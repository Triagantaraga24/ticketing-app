#[macro_use] extern crate rocket;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

use dotenvy::dotenv;

mod config;
mod db;
mod models;
mod routes;
mod utils;

use config::Config;
use db::init_db;
use routes::{public, admin};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS, PUT, DELETE"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let config = Config::from_env();
    let db = init_db(&config).await;

    rocket::build()
        .manage(config)
        .manage(db)
        .attach(CORS)
        .mount("/api", public::routes())
        .mount("/api/admin", admin::routes())
}