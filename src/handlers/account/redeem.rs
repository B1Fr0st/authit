use actix_web::{HttpResponse, web};
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::JwtClaims;

#[derive(Deserialize)]
pub struct RedeemRequest {
    key: String,
}

#[derive(Serialize)]
pub struct RedeemResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}


async fn key_db_query(pool: &sqlx::PgPool, key: &str) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>("SELECT time_hours, product_id FROM cd_keys WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
}

async fn user_products_query(pool: &sqlx::PgPool, user_id: &str) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String,)>("SELECT product_id FROM user_licenses WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|row| row.0).collect())
}

async fn user_product_extend_query(pool: &sqlx::PgPool, user_id: &str, product_id: &str, extend_by_hours: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE user_licenses SET expires_at = expires_at + ($1 || ' hours')::INTERVAL, updated_at = NOW() WHERE user_id = $2 AND product_id = $3")
        .bind(extend_by_hours)
        .bind(user_id)
        .bind(product_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn user_product_assign_query(pool: &sqlx::PgPool, user_id: &str, product_id: &str, hours: i64) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO user_licenses (user_id, product_id, expires_at) VALUES ($1, $2, NOW() + ($3 || ' hours')::INTERVAL)")
        .bind(user_id)
        .bind(product_id)
        .bind(hours)
        .execute(pool)
        .await?;

    Ok(())
}

async fn consume_key_query(pool: &sqlx::PgPool, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM cd_keys WHERE key = $1")
        .bind(key)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn redeem(
    claims: JwtClaims,
    body: web::Json<RedeemRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    //redeem flow:
    /*
        - verify jwt token
        - check if the key exists and is valid/unused
        - check if the user has that product already
        - if user doesn't have product, assign product to user
        - if they have the product, add however much time the key gives to their existing product license
        - consume key
        - respond with success or failure message
     */
    info!("Redeem attempt for key: {} on userid {}", body.key, claims.sub);

    //validate key
    let (time_hours, product_id) = match key_db_query(&data.db_pool, &body.key).await {
        Ok(Some((time_hours, product_id))) => (time_hours, product_id),
        Ok(None) => {
            info!("Redeem failed: invalid key {}", body.key);
            return HttpResponse::Ok().json(RedeemResponse {
                success: false,
                message: Some("Invalid or already used key.".to_string()),
            });
        }
        Err(err) => {
            error!("Database error during key lookup: {}", err);
            return HttpResponse::InternalServerError().json(RedeemResponse {
                success: false,
                message: Some("Internal server error.".to_string()),
            });
        }
    };
    info!("Key valid for product {} with {} hours", product_id, time_hours);

    //get user's current products/licenses & check if they have the product
    let products = match user_products_query(&data.db_pool, &claims.sub).await {
        Ok(products) => products,
        Err(err) => {
            error!("Database error during user products lookup: {}", err);
            return HttpResponse::InternalServerError().json(RedeemResponse {
                success: false,
                message: Some("Internal server error.".to_string()),
            });
        }
    };
    info!("User {} currently has products: {:?}", claims.sub, products);

    // Assign or extend license
    if products.contains(&product_id) {
        info!("User {} already owns product {}, extending license by {} hours", claims.sub, product_id, time_hours);

        if let Err(err) = user_product_extend_query(&data.db_pool, &claims.sub, &product_id, time_hours).await {
            error!("Database error during license extension: {}", err);
            return HttpResponse::InternalServerError().json(RedeemResponse {
                success: false,
                message: Some("Failed to extend license.".to_string()),
            });
        }
    } else {
        info!("Assigning product {} to user {} with {} hours", product_id, claims.sub, time_hours);

        if let Err(err) = user_product_assign_query(&data.db_pool, &claims.sub, &product_id, time_hours).await {
            error!("Database error during product assignment: {}", err);
            return HttpResponse::InternalServerError().json(RedeemResponse {
                success: false,
                message: Some("Failed to assign product.".to_string()),
            });
        }
    }

    // Consume the key (delete it from database)
    if let Err(err) = consume_key_query(&data.db_pool, &body.key).await {
        error!("Database error during key consumption: {}", err);
        // Note: License was already assigned/extended, so we still return success
        // but log the error for investigation
        error!("CRITICAL: Key {} was used but not consumed from database!", body.key);
    }

    info!("Successfully redeemed key {} for user {}", body.key, claims.sub);

    // Convert hours to days for user-friendly message
    let time_days = time_hours / 24;
    let remaining_hours = time_hours % 24;

    let time_message = if remaining_hours == 0 {
        format!("{} days", time_days)
    } else {
        format!("{} days and {} hours", time_days, remaining_hours)
    };

    HttpResponse::Ok().json(RedeemResponse {
        success: true,
        message: Some(format!("Successfully redeemed {} for product {}.", time_message, product_id)),
    })
}