use poem::{handler, web::Data, IntoResponse, Request, Result};
use crate::db::DbPool;
use crate::db::license::LicenseDb;
use crate::db::product::ProductDb;
use crate::data::LoginData;
use super::auth::extract_and_validate_auth;

#[handler]
pub async fn get_licenses(
    req: &Request,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Fetching all licenses");

    // Get all licenses
    let licenses = LicenseDb::get_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch licenses: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    tracing::info!("Retrieved {} licenses", licenses.len());

    Ok(poem::web::Json(licenses))
}

#[handler]
pub async fn get_products(
    req: &Request,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Fetching all products");

    // Get all products
    let products = ProductDb::get_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch products: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    tracing::info!("Retrieved {} products", products.len());

    Ok(poem::web::Json(products))
}

#[handler]
pub async fn get_logins(
    req: &Request,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    // Get query parameter for limit (default to 100)
    let limit = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .find(|pair| pair.starts_with("limit="))
                .and_then(|pair| pair.split('=').nth(1))
                .and_then(|v| v.parse::<i64>().ok())
        })
        .unwrap_or(100);

    tracing::info!("Fetching login logs (limit: {})", limit);

    // Get recent login logs
    let logs = LoginData::get_recent_logs(pool, limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch login logs: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    tracing::info!("Retrieved {} login log entries", logs.len());

    Ok(poem::web::Json(logs))
}

#[handler]
pub async fn get_logs(
    req: &Request,
    Data(_pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    // Get query parameter for limit (default to 1000)
    let limit = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .find(|pair| pair.starts_with("limit="))
                .and_then(|pair| pair.split('=').nth(1))
                .and_then(|v| v.parse::<usize>().ok())
        })
        .unwrap_or(1000);

    // Get logs from the global log store
    let logs = crate::logging::get_log_store()
        .map(|store| store.get_logs(limit))
        .unwrap_or_else(Vec::new);

    tracing::info!("Retrieved {} log entries", logs.len());

    Ok(poem::web::Json(logs))
}
