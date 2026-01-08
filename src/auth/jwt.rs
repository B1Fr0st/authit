use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use actix_web::{FromRequest, HttpRequest, HttpResponse, dev::Payload, error::ResponseError, web};
use std::pin::Pin;
use std::fmt;
use std::ops::Deref;
use std::future::Future;

use crate::handlers::account::Role;
use crate::AppState;
use super::blacklist::TokenBlacklist;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user id)
    pub email: String,
    pub role: Role,
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at
}

impl Claims {
    pub fn new(user_id: String, email: String, role: Role) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        Self {
            sub: user_id,
            email,
            role,
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        }
    }
}

pub fn generate_token(user_id: String, email: String, role: Role) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, email, role);
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-insecure-secret".to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-insecure-secret".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

// Custom error type for JWT authentication
#[derive(Debug)]
pub enum JwtError {
    Missing,
    Invalid,
    Expired,
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtError::Missing => write!(f, "Missing authorization token"),
            JwtError::Invalid => write!(f, "Invalid token"),
            JwtError::Expired => write!(f, "Token expired"),
        }
    }
}

impl ResponseError for JwtError {
    fn error_response(&self) -> HttpResponse {
        match self {
            JwtError::Missing => HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Missing authorization token"
            })),
            JwtError::Invalid => HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Invalid token"
            })),
            JwtError::Expired => HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Token expired"
            })),
        }
    }
}

// Actix-web extractor for JWT claims
// Usage in routes: async fn handler(claims: JwtClaims) -> impl Responder
#[derive(Debug, Clone)]
pub struct JwtClaims(pub Claims);

impl Deref for JwtClaims {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for JwtClaims {
    type Error = JwtError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let auth_header = req.headers().get("Authorization");

            let token = match auth_header {
                Some(header_value) => {
                    match header_value.to_str() {
                        Ok(header_str) => {
                            // Support both "Bearer <token>" and just "<token>"
                            if header_str.starts_with("Bearer ") {
                                header_str[7..].to_string()
                            } else {
                                header_str.to_string()
                            }
                        }
                        Err(_) => return Err(JwtError::Invalid),
                    }
                }
                None => return Err(JwtError::Missing),
            };

            // Decode and validate the token
            let claims = match decode_token(&token) {
                Ok(claims) => claims,
                Err(err) => {
                    use jsonwebtoken::errors::ErrorKind;
                    return Err(match err.kind() {
                        ErrorKind::ExpiredSignature => JwtError::Expired,
                        _ => JwtError::Invalid,
                    });
                }
            };

            // Check blacklist if Redis is available
            if let Some(app_state) = req.app_data::<web::Data<AppState>>() {
                let blacklist = TokenBlacklist::new(app_state.redis_client.clone());

                // Check if specific token is blacklisted
                if blacklist.is_token_blacklisted(&token).await.unwrap_or(false) {
                    return Err(JwtError::Invalid);
                }

                // Check if token was issued before user's role change (or other invalidation event)
                if blacklist.is_user_token_blacklisted(&claims.sub, claims.iat).await.unwrap_or(false) {
                    return Err(JwtError::Invalid);
                }
            }

            Ok(JwtClaims(claims))
        })
    }
}
