use poem::{handler, web::{Data, Json}, IntoResponse, Request, Result};
use crate::db::DbPool;
use crate::types::requests::{GeneratorRequest, GeneratorResponse};
use crate::types::core::{License, LicenseProduct};
use crate::db::license::LicenseDb;
use crate::db::product::ProductDb;
use super::auth::extract_and_validate_auth;
use std::sync::Arc;
use rand::Rng;

#[handler]
pub async fn generator(
    req: &Request,
    Json(request): Json<GeneratorRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    // Generate license
    let response = generate_license(pool, &request).await;
    Ok(Json(response))
}

// Business logic for license generation
async fn generate_license(
    pool: &DbPool,
    request: &GeneratorRequest,
) -> GeneratorResponse {
    // 1. Validate all products exist
    for product_id in request.products.keys() {
        match ProductDb::exists(pool, product_id).await {
            Ok(true) => {},
            _ => return GeneratorResponse::OneOrMoreInvalidProduct,
        }
    }

    // 2. Generate a unique license key (retry up to 10 times)
    let license_key = match generate_unique_license_key(pool).await {
        Some(key) => key,
        None => return GeneratorResponse::FailedToGenerateValidLicense,
    };

    // 3. Get current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 4. Create license products
    let mut products = Vec::new();
    for (product_id, time) in &request.products {
        products.push(LicenseProduct {
            product: Arc::from(product_id.as_str()),
            time: *time,
            started_at: now,
        });
    }

    // 5. Create the license
    let license = License {
        license_key: Arc::from(license_key.as_str()),
        products,
        hwid: Arc::from(""), // HWID is set on first auth
        sessions: Vec::new(),
    };

    // 6. Store in database
    match LicenseDb::create(pool, license).await {
        Ok(_) => GeneratorResponse::Ok(license_key),
        Err(_) => GeneratorResponse::FailedToGenerateValidLicense,
    }
}

// Generate a unique license key
async fn generate_unique_license_key(pool: &DbPool) -> Option<String> {
    for _ in 0..10 {
        let key = generate_random_key();

        match LicenseDb::exists(pool, &key).await {
            Ok(false) => return Some(key),
            _ => continue,
        }
    }
    None
}

// Generate a random license key in format: XXXXX-XXXXX-XXXXX
fn generate_random_key() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();

    let part1: String = (0..5).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
    let part2: String = (0..5).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
    let part3: String = (0..5).map(|_| chars[rng.gen_range(0..chars.len())]).collect();

    format!("{}-{}-{}", part1, part2, part3)
}
