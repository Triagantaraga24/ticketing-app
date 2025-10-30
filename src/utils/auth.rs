use jsonwebtoken::{encode, Header, EncodingKey, decode, Validation, DecodingKey};
use serde::{Serialize, Deserialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::http::Status;
use rocket::request::{Request, FromRequest, Outcome};
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
}

#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub fn create_jwt(email: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: email.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

#[derive(Debug, Clone)]
pub struct AdminAuth {
    pub email: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Ambil JWT secret dari state Config
        let config = match request.rocket().state::<Config>() {
            Some(c) => c,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        if let Some(auth_header) = request.headers().get_one("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
                if let Ok(token_data) = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(config.jwt_secret.as_ref()),
                    &validation,
                ) {
                    return Outcome::Success(AdminAuth {
                        email: token_data.claims.sub,
                    });
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}