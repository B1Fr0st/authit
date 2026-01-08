use actix_web::{HttpResponse, web};
use tracing::{error, info};
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::AppState;
use crate::auth::JwtClaims;
use crate::handlers::account::Role;

#[derive(Deserialize)]
pub struct GenerateKeyRequest {
    product_id: String,
    time_days: i64,
    #[serde(default = "default_count")]
    count: i32,
}

fn default_count() -> i32 {
    1
}

#[derive(Serialize)]
pub struct GenerateKeyResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Generate a random CD key in format: XXXX-XXXX-XXXX-XXXX
fn generate_random_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    let mut key = String::with_capacity(50) + std::env::var("KEY_PREFIX").unwrap_or_else(|_| "".to_string()).as_str(); // 16 chars + 3 dashes

    for segment in 0..7 {
        if segment > 0 {
            key.push('-');
        }
        for _ in 0..5 {
            let idx = rng.gen_range(0..CHARSET.len());
            key.push(CHARSET[idx] as char);
        }
    }

    key
}

/// Check if a product exists in the database
async fn product_exists(pool: &sqlx::PgPool, product_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("SELECT id FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_optional(pool)
        .await?;

    Ok(result.is_some())
}

/// Insert a CD key into the database
async fn insert_key(pool: &sqlx::PgPool, key: &str, product_id: &str, time_hours: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO cd_keys (key, product_id, time_hours) VALUES ($1, $2, $3) ON CONFLICT (key) DO NOTHING"
    )
    .bind(key)
    .bind(product_id)
    .bind(time_hours)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn generate_key(
    claims: JwtClaims,
    body: web::Json<GenerateKeyRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    info!("Generate key attempt by {} for product {} ({} days, count: {})",
          claims.sub, body.product_id, body.time_days, body.count);

    // Check if requester is Admin
    if !matches!(claims.role, Role::Admin) {
        info!("Generate key denied: user {} is not an admin (role: {:?})", claims.sub, claims.role);
        return HttpResponse::Forbidden().json(GenerateKeyResponse {
            success: false,
            keys: None,
            message: Some("Only admins can generate CD keys.".to_string()),
        });
    }

    // Validate time_days is positive
    if body.time_days <= 0 {
        info!("Generate key denied: invalid time_days {}", body.time_days);
        return HttpResponse::BadRequest().json(GenerateKeyResponse {
            success: false,
            keys: None,
            message: Some("time_days must be positive.".to_string()),
        });
    }

    // Convert days to hours for storage
    let time_hours = body.time_days * 24;

    // Validate count is positive and reasonable
    if body.count <= 0 || body.count > 1000 {
        info!("Generate key denied: invalid count {}", body.count);
        return HttpResponse::BadRequest().json(GenerateKeyResponse {
            success: false,
            keys: None,
            message: Some("count must be between 1 and 1000.".to_string()),
        });
    }

    // Check if product exists
    match product_exists(&data.db_pool, &body.product_id).await {
        Ok(exists) => {
            if !exists {
                info!("Generate key failed: product {} does not exist", body.product_id);
                return HttpResponse::NotFound().json(GenerateKeyResponse {
                    success: false,
                    keys: None,
                    message: Some("Product not found.".to_string()),
                });
            }
        }
        Err(err) => {
            error!("Database error checking product existence: {}", err);
            return HttpResponse::InternalServerError().json(GenerateKeyResponse {
                success: false,
                keys: None,
                message: Some("Internal server error.".to_string()),
            });
        }
    }

    // Generate keys
    let mut generated_keys = Vec::new();
    let mut attempts = 0;
    const MAX_ATTEMPTS_PER_KEY: i32 = 10;

    while generated_keys.len() < body.count as usize {
        if attempts >= body.count * MAX_ATTEMPTS_PER_KEY {
            error!("Failed to generate {} keys after {} attempts", body.count, attempts);
            let count = generated_keys.len();
            return HttpResponse::InternalServerError().json(GenerateKeyResponse {
                success: false,
                keys: Some(generated_keys),
                message: Some(format!("Only generated {} out of {} keys due to collisions.",
                                    count, body.count)),
            });
        }

        let key = generate_random_key();
        attempts += 1;

        match insert_key(&data.db_pool, &key, &body.product_id, time_hours).await {
            Ok(inserted) => {
                if inserted {
                    generated_keys.push(key.clone());
                    info!("Generated key: {}", key);
                } else {
                    // Key collision, try again
                    info!("Key collision for {}, retrying...", key);
                }
            }
            Err(err) => {
                error!("Database error inserting key: {}", err);
                let count = generated_keys.len();
                return HttpResponse::InternalServerError().json(GenerateKeyResponse {
                    success: false,
                    keys: Some(generated_keys),
                    message: Some(format!("Partial success: generated {} keys before error.",
                                        count)),
                });
            }
        }
    }

    info!("Successfully generated {} keys for product {}", generated_keys.len(), body.product_id);
    HttpResponse::Ok().json(GenerateKeyResponse {
        success: true,
        keys: Some(generated_keys),
        message: Some(format!("Successfully generated {} key(s).", body.count)),
    })
}