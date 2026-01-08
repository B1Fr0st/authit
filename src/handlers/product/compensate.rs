use actix_web::{HttpResponse, web};
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::JwtClaims;
use crate::handlers::account::Role;

#[derive(Deserialize)]
pub struct CompensateRequest {
    product_id: String,
    time_hours: i64,
}

#[derive(Serialize)]
pub struct CompensateResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    users_compensated: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Check if a product exists in the database
async fn product_exists(pool: &sqlx::PgPool, product_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("SELECT id FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_optional(pool)
        .await?;

    Ok(result.is_some())
}

/// Extend licenses for all users who have the specified product
async fn extend_all_user_licenses(pool: &sqlx::PgPool, product_id: &str, time_hours: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE user_licenses
         SET expires_at = expires_at + ($1 || ' hours')::INTERVAL,
             updated_at = NOW()
         WHERE product_id = $2"
    )
    .bind(time_hours)
    .bind(product_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn compensate(
    claims: JwtClaims,
    body: web::Json<CompensateRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    info!("Compensate attempt by {} for product {} ({} hours)",
          claims.sub, body.product_id, body.time_hours);

    // Check if requester is Admin
    if !matches!(claims.role, Role::Admin) {
        info!("Compensate denied: user {} is not an admin (role: {:?})", claims.sub, claims.role);
        return HttpResponse::Forbidden().json(CompensateResponse {
            success: false,
            users_compensated: None,
            message: Some("Only admins can compensate users.".to_string()),
        });
    }

    // Validate time_hours is positive
    if body.time_hours <= 0 {
        info!("Compensate denied: invalid time_hours {}", body.time_hours);
        return HttpResponse::BadRequest().json(CompensateResponse {
            success: false,
            users_compensated: None,
            message: Some("time_hours must be positive.".to_string()),
        });
    }

    // Check if product exists
    match product_exists(&data.db_pool, &body.product_id).await {
        Ok(exists) => {
            if !exists {
                info!("Compensate failed: product {} does not exist", body.product_id);
                return HttpResponse::NotFound().json(CompensateResponse {
                    success: false,
                    users_compensated: None,
                    message: Some("Product not found.".to_string()),
                });
            }
        }
        Err(err) => {
            error!("Database error checking product existence: {}", err);
            return HttpResponse::InternalServerError().json(CompensateResponse {
                success: false,
                users_compensated: None,
                message: Some("Internal server error.".to_string()),
            });
        }
    }

    // Extend all user licenses for this product
    match extend_all_user_licenses(&data.db_pool, &body.product_id, body.time_hours).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                info!("Compensate completed but no users found with product {}", body.product_id);
                return HttpResponse::Ok().json(CompensateResponse {
                    success: true,
                    users_compensated: Some(0),
                    message: Some("No users have this product.".to_string()),
                });
            }

            info!("Successfully compensated {} users with {} hours for product {}",
                  rows_affected, body.time_hours, body.product_id);
            HttpResponse::Ok().json(CompensateResponse {
                success: true,
                users_compensated: Some(rows_affected as i32),
                message: Some(format!("Successfully extended licenses for {} user(s) by {} hours.",
                                    rows_affected, body.time_hours)),
            })
        }
        Err(err) => {
            error!("Database error during compensation: {}", err);
            HttpResponse::InternalServerError().json(CompensateResponse {
                success: false,
                users_compensated: None,
                message: Some("Internal server error.".to_string()),
            })
        }
    }
}