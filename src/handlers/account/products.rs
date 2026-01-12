use actix_web::{HttpResponse, web};
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::JwtClaims;

#[derive(Serialize)]
pub struct ProductLicense {
    product_id: String,
    product_name: String,
    expires_at: String,
    time_remaining_seconds: i64,
    frozen: bool,
}

#[derive(Serialize)]
pub struct ProductsResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    products: Option<Vec<ProductLicense>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

async fn get_user_products(
    pool: &sqlx::PgPool,
    user_id: &str,
) -> Result<Vec<ProductLicense>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, i64, bool)>(
        "SELECT ul.product_id, p.name, ul.expires_at::TEXT, EXTRACT(EPOCH FROM (ul.expires_at - NOW()))::BIGINT, p.frozen
         FROM user_licenses ul
         JOIN products p ON ul.product_id = p.id
         WHERE ul.user_id = $1 AND ul.expires_at > NOW()
         ORDER BY ul.expires_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(product_id, product_name, expires_at, time_remaining_seconds, frozen)| ProductLicense {
            product_id,
            product_name,
            expires_at,
            time_remaining_seconds,
            frozen,
        })
        .collect())
}

async fn get_all_products_lifetime(
    pool: &sqlx::PgPool,
) -> Result<Vec<ProductLicense>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT id, name, frozen FROM products ORDER BY name"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(product_id, product_name, frozen)| ProductLicense {
            product_id,
            product_name,
            expires_at: "infinity".to_string(),
            time_remaining_seconds: i64::MAX,
            frozen,
        })
        .collect())
}

pub async fn products(
    claims: JwtClaims,
    data: web::Data<AppState>,
) -> HttpResponse {
    info!("Products request for user {}", claims.sub);

    // Admins and Devs get lifetime access to all products
    if matches!(claims.role, crate::handlers::account::Role::Admin | crate::handlers::account::Role::Dev) {
        info!("Admin/Dev user {} requesting products - returning all products with lifetime access", claims.sub);

        match get_all_products_lifetime(&data.db_pool).await {
            Ok(products) => {
                return HttpResponse::Ok().json(ProductsResponse {
                    success: true,
                    products: Some(products),
                    message: Some("Lifetime access to all products.".to_string()),
                });
            }
            Err(err) => {
                error!("Database error while fetching all products: {}", err);
                return HttpResponse::InternalServerError().json(ProductsResponse {
                    success: false,
                    products: None,
                    message: Some("Internal server error.".to_string()),
                });
            }
        }
    }

    match get_user_products(&data.db_pool, &claims.sub).await {
        Ok(products) => {
            if products.is_empty() {
                info!("User {} has no active products", claims.sub);
                HttpResponse::Ok().json(ProductsResponse {
                    success: true,
                    products: Some(vec![]),
                    message: Some("No active products found.".to_string()),
                })
            } else {
                info!("User {} has {} active product(s)", claims.sub, products.len());
                HttpResponse::Ok().json(ProductsResponse {
                    success: true,
                    products: Some(products),
                    message: None,
                })
            }
        }
        Err(err) => {
            error!("Database error while fetching products for user {}: {}", claims.sub, err);
            HttpResponse::InternalServerError().json(ProductsResponse {
                success: false,
                products: None,
                message: Some("Internal server error.".to_string()),
            })
        }
    }
}
