use actix_web::{HttpResponse, web};
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::JwtClaims;

#[derive(Deserialize)]
pub struct AuthRequest {
    product_id: String,
    hwid: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

type DbResponse = Result<Option<i64>, sqlx::Error>;
async fn get_license_time_remaining(
    pool: &sqlx::PgPool,
    user_id: &str,
    product_id: &str,
) -> DbResponse {
    sqlx::query_as::<_, (i64,)>(
        "SELECT EXTRACT(EPOCH FROM (expires_at - NOW()))::BIGINT FROM user_licenses WHERE user_id = $1 AND product_id = $2 AND expires_at > NOW()",
    )
    .bind(user_id)
    .bind(product_id)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(|row| row.0))
}

async fn check_hwid(
    pool: &sqlx::PgPool,
    user_id: &str,
    hwid: &str,
) -> Result<Option<bool>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT hwid FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((Some(stored_hwid),)) => Ok(Some(stored_hwid == hwid)),
        Some((None,)) => Ok(None), // No HWID set yet - needs auto-binding
        None => Ok(Some(false)), // User not found
    }
}

async fn bind_hwid(
    pool: &sqlx::PgPool,
    user_id: &str,
    hwid: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET hwid = $1, updated_at = NOW() WHERE id = $2 AND hwid IS NULL"
    )
    .bind(hwid)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

async fn check_if_banned(
    pool: &sqlx::PgPool,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_as::<_, (bool,)>(
        "SELECT banned FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((banned,)) => Ok(banned),
        None => Ok(false), // User not found
    }
}

async fn check_hwid_banned(
    pool: &sqlx::PgPool,
    hwid: &str,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT hwid FROM banned_hwids WHERE hwid = $1",
    )
    .bind(hwid)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
     

pub async fn auth(
    claims: JwtClaims,
    body: web::Json<AuthRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    // admins & devs always have access to all products
    if matches!(claims.role, crate::handlers::account::Role::Admin | crate::handlers::account::Role::Dev) {
        return HttpResponse::Ok().json(AuthResponse {
            success: true,
            time_remaining: Some(i64::MAX),
            message: None,
        });
    }

    match check_if_banned(&data.db_pool, &claims.sub).await {
        Ok(true) => {
            info!("Banned user {} attempted authentication", &claims.sub);
            return HttpResponse::Forbidden().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Your account has been banned. Contact support for more information.".to_string()),
            });
        },
        Ok(false) => {
            // not banned, continue
        }
        Err(err) => {
            error!("Database error while checking ban status for user {}: {}", &claims.sub, err);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Internal server error - contact support.".to_string()),
            });
        }
    }

    // Check if HWID is banned
    match check_hwid_banned(&data.db_pool, &body.hwid).await {
        Ok(true) => {
            info!("Banned HWID {} attempted authentication (user: {})", &body.hwid, &claims.sub);
            return HttpResponse::Forbidden().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Your hardware has been banned. Contact support for more information.".to_string()),
            });
        },
        Ok(false) => {
            // HWID not banned, continue
        }
        Err(err) => {
            error!("Database error while checking HWID ban for {}: {}", &body.hwid, err);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Internal server error - contact support.".to_string()),
            });
        }
    }

    match check_hwid(&data.db_pool, &claims.sub, &body.hwid).await {
        Ok(Some(true)) => {
            info!("HWID check passed for user {}", &claims.sub);
        },
        Ok(Some(false)) => {
            info!("HWID check failed for user {}", &claims.sub);
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("HWID mismatch. If you are on the same machine or recently changed your hardware, please contact support.".to_string()),
            });
        },
        Ok(None) => {
            // No HWID set yet - auto-bind it
            info!("No HWID set for user {}, attempting to bind HWID: {}", &claims.sub, &body.hwid);
            match bind_hwid(&data.db_pool, &claims.sub, &body.hwid).await {
                Ok(true) => {
                    info!("Successfully bound HWID for user {}", &claims.sub);
                },
                Ok(false) => {
                    error!("Failed to bind HWID for user {} - no rows affected", &claims.sub);
                    return HttpResponse::InternalServerError().json(AuthResponse {
                        success: false,
                        time_remaining: None,
                        message: Some("Failed to bind HWID - contact support.".to_string()),
                    });
                },
                Err(err) => {
                    error!("Database error while binding HWID for user {}: {}", &claims.sub, err);
                    return HttpResponse::InternalServerError().json(AuthResponse {
                        success: false,
                        time_remaining: None,
                        message: Some("Internal server error - contact support.".to_string()),
                    });
                }
            }
        },
        Err(err) => {
            error!("Database error while checking HWID for user {}: {}", &claims.sub, err);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Internal server error - contact support.".to_string()),
            });
        }
    }


    match get_license_time_remaining(&data.db_pool, &claims.sub, &body.product_id).await {
        Ok(Some(time)) => {
            info!("User {} authenticated for product {} with {} seconds remaining", &claims.sub, &body.product_id, time);
            return HttpResponse::Ok().json(AuthResponse {
                success: true,
                time_remaining: Some(time),
                message: Some(format!("Welcome back, {}.", &claims.sub.to_string())),
            });
        },
        Ok(None) => {
            return HttpResponse::Ok().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Product not found or expired.".to_string()),
            });
        }
        Err(err) => {
            error!("Database error while checking license for user {} and product {}: {}", &claims.sub, &body.product_id, err);
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                time_remaining: None,
                message: Some("Internal server error - contact support.".to_string()),
            });
        }
    }
}