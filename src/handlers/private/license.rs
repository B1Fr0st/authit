use poem::{handler, web::{Data, Json}, IntoResponse, Request, Result};
use crate::db::DbPool;
use crate::types::requests::{
    GeneratorRequest, GeneratorResponse,
    AddProductRequest, AddProductResponse,
    DeleteProductRequest, DeleteProductResponse,
};
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

    tracing::info!("License generation request with {} products", request.products.len());

    // Generate license
    let response = generate_license(pool, &request).await;

    match &response {
        GeneratorResponse::Ok(key) => {
            tracing::info!("Successfully generated license: {}", key);
        },
        GeneratorResponse::OneOrMoreInvalidProduct => {
            tracing::warn!("License generation failed: invalid product(s)");
        },
        GeneratorResponse::FailedToGenerateValidLicense => {
            tracing::error!("License generation failed: unable to generate unique key");
        },
    }

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

#[handler]
pub async fn add_product(
    req: &Request,
    Json(request): Json<AddProductRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Add product request for license: {} with {} products",
        request.license, request.products.len());

    let response = add_products_to_license(pool, &request).await;

    match &response {
        AddProductResponse::Ok => {
            tracing::info!("Successfully added products to license: {}", request.license);
        },
        AddProductResponse::InvalidLicense => {
            tracing::warn!("Failed to add products: license {} not found", request.license);
        },
        AddProductResponse::OneOrMoreInvalidProduct => {
            tracing::warn!("Failed to add products: one or more invalid products");
        },
    }

    Ok(Json(response))
}

#[handler]
pub async fn delete_product(
    req: &Request,
    Json(request): Json<DeleteProductRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Delete product request for license: {} with {} products",
        request.license, request.products.len());

    let response = delete_products_from_license(pool, &request).await;

    match &response {
        DeleteProductResponse::Ok => {
            tracing::info!("Successfully deleted products from license: {}", request.license);
        },
        DeleteProductResponse::InvalidLicense => {
            tracing::warn!("Failed to delete products: license {} not found", request.license);
        },
        DeleteProductResponse::OneOrMoreInvalidProduct => {
            tracing::warn!("Failed to delete products: one or more invalid products");
        },
    }

    Ok(Json(response))
}

// Business logic for adding products to a license
async fn add_products_to_license(
    pool: &DbPool,
    request: &AddProductRequest,
) -> AddProductResponse {
    // 1. Check if license exists
    match LicenseDb::exists(pool, &request.license).await {
        Ok(true) => {},
        _ => return AddProductResponse::InvalidLicense,
    }

    // 2. Validate all products exist
    for product_id in request.products.keys() {
        match ProductDb::exists(pool, product_id).await {
            Ok(true) => {},
            _ => return AddProductResponse::OneOrMoreInvalidProduct,
        }
    }

    // 3. Get current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 4. Add each product to the license
    for (product_id, time) in &request.products {
        match ProductDb::add_to_license(pool, &request.license, product_id, *time, now).await {
            Ok(_) => {},
            Err(_) => return AddProductResponse::OneOrMoreInvalidProduct,
        }
    }

    AddProductResponse::Ok
}

// Business logic for deleting products from a license
async fn delete_products_from_license(
    pool: &DbPool,
    request: &DeleteProductRequest,
) -> DeleteProductResponse {
    // 1. Check if license exists
    match LicenseDb::exists(pool, &request.license).await {
        Ok(true) => {},
        _ => return DeleteProductResponse::InvalidLicense,
    }

    // 2. Validate all products exist
    for product_id in &request.products {
        match ProductDb::exists(pool, product_id).await {
            Ok(true) => {},
            _ => return DeleteProductResponse::OneOrMoreInvalidProduct,
        }
    }

    // 3. Remove each product from the license
    for product_id in &request.products {
        match ProductDb::remove_from_license(pool, &request.license, product_id).await {
            Ok(_) => {},
            Err(_) => return DeleteProductResponse::OneOrMoreInvalidProduct,
        }
    }

    DeleteProductResponse::Ok
}
