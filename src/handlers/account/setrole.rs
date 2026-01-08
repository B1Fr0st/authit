use actix_web::{HttpResponse, web};
use tracing::{error, info};
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::AppState;
use crate::auth::{JwtClaims, TokenBlacklist};
use super::Role;

#[derive(Deserialize)]
pub struct SetRoleRequest {
    user_id: String,
    role: Role,
}

#[derive(Serialize)]
pub struct SetRoleResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

async fn update_user_role_query(pool: &sqlx::PgPool, user_id: &str, role: Role) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2")
        .bind(role)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn set_role(
    claims: JwtClaims,
    body: web::Json<SetRoleRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    info!("SetRole attempt by {} for user {} to role {:?}", claims.sub, body.user_id, body.role);

    // Check if requester is Admin
    if !matches!(claims.role, Role::Admin) {
        info!("SetRole denied: user {} is not an admin (role: {:?})", claims.sub, claims.role);
        return HttpResponse::Forbidden().json(SetRoleResponse {
            success: false,
            message: Some("Only admins can change user roles.".to_string()),
        });
    }

    // Prevent self-demotion
    if claims.sub == body.user_id && !matches!(body.role, Role::Admin) {
        info!("SetRole denied: admin {} attempted to demote themselves", claims.sub);
        return HttpResponse::BadRequest().json(SetRoleResponse {
            success: false,
            message: Some("Admins cannot demote themselves.".to_string()),
        });
    }

    // Update user role
    match update_user_role_query(&data.db_pool, &body.user_id, body.role).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                info!("SetRole failed: user {} not found", body.user_id);
                return HttpResponse::NotFound().json(SetRoleResponse {
                    success: false,
                    message: Some("User not found.".to_string()),
                });
            }

            // Blacklist all tokens issued before now (24 hours = max token lifetime)
            let now = Utc::now().timestamp();
            let blacklist = TokenBlacklist::new(data.redis_client.clone());
            if let Err(e) = blacklist.blacklist_user_before_timestamp(&body.user_id, now, 86400).await {
                error!("Failed to blacklist user tokens: {}", e);
                // Continue anyway - role was updated in database
            }

            info!("Successfully updated user {} to role {:?} and invalidated all tokens", body.user_id, body.role);
            HttpResponse::Ok().json(SetRoleResponse {
                success: true,
                message: Some(format!("Successfully updated user role to {:?}. User must re-login.", body.role)),
            })
        }
        Err(err) => {
            error!("Database error during role update: {}", err);
            HttpResponse::InternalServerError().json(SetRoleResponse {
                success: false,
                message: Some("Internal server error.".to_string()),
            })
        }
    }
}
