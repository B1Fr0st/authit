use actix_web::{HttpResponse, web};
use tracing::{error, info};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::jwt;
use super::Role;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

type DbResponse = Result<Option<(String, String, crate::handlers::account::Role)>, sqlx::Error>;

async fn db_query(pool: &sqlx::PgPool, email: &str) -> DbResponse {
    sqlx::query_as::<_, (String, String, Role)>("SELECT id, password, role FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn login(
    body: web::Json<LoginRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    info!("Login attempt for email: {}", body.email);

    // Fetch user from database by email
    let response = db_query(&data.db_pool, &body.email).await;

    match response {
        Ok(Some((user_id, password_hash, role))) => {
            // Verify password using argon2
            let argon2 = Argon2::default();
            match PasswordHash::new(&password_hash) {
                Ok(parsed_hash) => {
                    match argon2.verify_password(body.password.as_bytes(), &parsed_hash) {
                        Ok(_) => {
                            info!("Login successful for user: {}", user_id);

                            // Generate JWT token
                            match jwt::generate_token(user_id, body.email.clone(), role) {
                                Ok(token) => {
                                    HttpResponse::Ok().json(LoginResponse {
                                        success: true,
                                        token: Some(token),
                                        message: None,
                                    })
                                }
                                Err(e) => {
                                    error!("Failed to generate JWT token: {}", e);
                                    HttpResponse::InternalServerError().json(LoginResponse {
                                        success: false,
                                        token: None,
                                        message: Some("Failed to generate token".to_string()),
                                    })
                                }
                            }
                        }
                        Err(_) => {
                            info!("Invalid password for email: {}", body.email);
                            HttpResponse::Unauthorized().json(LoginResponse {
                                success: false,
                                token: None,
                                message: Some("Invalid credentials".to_string()),
                            })
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse password hash: {}", e);
                    HttpResponse::InternalServerError().json(LoginResponse {
                        success: false,
                        token: None,
                        message: Some("Internal server error".to_string()),
                    })
                }
            }
        }
        Ok(None) => {
            info!("No user found with email: {}", body.email);
            HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                token: None,
                message: Some("Invalid credentials".to_string()),
            })
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                token: None,
                message: Some("Database error".to_string()),
            })
        }
    }
}