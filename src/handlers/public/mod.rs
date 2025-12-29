use poem::{handler, http::StatusCode, web::Data, IntoResponse, Request};
use crate::db::DbPool;
use crate::types::requests::{AuthHeaders, LoginResponse, ProductHeaders, ProductResponse};
use crate::data::LoginData;
use crate::db::{license::LicenseDb, product::ProductDb};

#[handler]
pub fn health() -> StatusCode {
    StatusCode::OK
}

#[handler]
pub async fn auth(
    req: &Request,
    Data(pool): Data<&DbPool>,
) -> impl IntoResponse {
    // Extract headers
    let auth_data = match extract_auth_headers(req) {
        Some(data) => {
            tracing::debug!("Auth request for license: {}, product: {}", data.license, data.product);
            data
        },
        None => {
            tracing::warn!("Auth request missing required headers");
            return poem::web::Json(LoginResponse::MissingHeaders);
        }
    };

    // Get current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Perform authorization logic
    let response = authorize_license(pool, &auth_data, now).await;

    tracing::info!("Auth attempt for license {}: {:?}", auth_data.license, response);

    // Log the login attempt
    let _ = LoginData::log_login(pool, now, &auth_data, response).await;

    poem::web::Json(response)
}

#[handler]
pub async fn product_handler(
    req: &Request,
    Data(pool): Data<&DbPool>,
) -> impl IntoResponse {
    // Extract headers
    let product_data = match extract_product_headers(req) {
        Some(data) => data,
        None => return poem::web::Json(ProductResponse::MissingHeaders),
    };

    let response = get_product_info(pool, &product_data).await;
    poem::web::Json(response)
}

// Extract auth headers from request
// Headers: X-License-Key, X-Product-ID, X-HWID
fn extract_auth_headers(req: &Request) -> Option<AuthHeaders> {
    let license = req.headers().get("X-License-Key")?.to_str().ok()?.to_string();
    let product = req.headers().get("X-Product-ID")?.to_str().ok()?.to_string();
    let hwid = req.headers().get("X-HWID")?.to_str().ok()?.to_string();

    Some(AuthHeaders {
        license,
        product,
        hwid,
    })
}

// Extract product headers from request
// Headers: X-License-Key, X-Product-ID
fn extract_product_headers(req: &Request) -> Option<ProductHeaders> {
    let license = req.headers().get("X-License-Key")?.to_str().ok()?.to_string();
    let product = req.headers().get("X-Product-ID")?.to_str().ok()?.to_string();

    Some(ProductHeaders {
        license,
        product,
    })
}

// Business logic for /auth endpoint
async fn authorize_license(
    pool: &DbPool,
    auth_data: &AuthHeaders,
    now: u64,
) -> LoginResponse {
    // 1. Check if license exists
    let license = match LicenseDb::get(pool, &auth_data.license).await {
        Ok(Some(license)) => license,
        _ => return LoginResponse::InvalidLicense,
    };

    // 2. Check HWID match (first time sets HWID, subsequent must match)
    if license.hwid.is_empty() {
        // First login - set HWID
        let _ = LicenseDb::update_hwid(pool, &auth_data.license, &auth_data.hwid).await;
    } else if license.hwid.as_ref() != auth_data.hwid.as_str() {
        return LoginResponse::HWIDMismatch;
    }

    // 3. Find the requested product in the license
    let license_product = match license.products.iter().find(|p| p.product.as_ref() == auth_data.product.as_str()) {
        Some(lp) => lp,
        None => return LoginResponse::InvalidLicense,
    };

    // 4. Check if product exists and is not frozen
    let prod = match ProductDb::get_by_id(pool, &auth_data.product).await {
        Ok(Some(p)) => p,
        _ => return LoginResponse::InvalidLicense,
    };

    if prod.frozen {
        return LoginResponse::LicenseFrozen;
    }

    // 5. Calculate time elapsed and check expiration
    let elapsed = now.saturating_sub(license_product.started_at);
    if elapsed >= license_product.time {
        return LoginResponse::LicenseExpired;
    }

    LoginResponse::Ok
}

// Business logic for /product endpoint
async fn get_product_info(
    pool: &DbPool,
    headers: &ProductHeaders,
) -> ProductResponse {
    // 1. Check if license exists
    let license = match LicenseDb::get(pool, &headers.license).await {
        Ok(Some(license)) => license,
        _ => return ProductResponse::InvalidLicense,
    };

    // 2. Find the requested product in the license
    let license_product = match license.products.iter().find(|p| p.product.as_ref() == headers.product.as_str()) {
        Some(lp) => lp,
        None => return ProductResponse::InvalidProduct,
    };

    ProductResponse::Ok(license_product.clone())
}