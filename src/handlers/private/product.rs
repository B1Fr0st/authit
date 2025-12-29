use poem::{handler, web::{Data, Json}, IntoResponse, Request, Result};
use crate::db::DbPool;
use crate::types::requests::{
    FreezeProductRequest, FreezeProductResponse,
    CreateProductRequest, CreateProductResponse,
    DeleteProductFromSystemRequest, DeleteProductFromSystemResponse,
};
use crate::db::product::ProductDb;
use super::auth::extract_and_validate_auth;

#[handler]
pub async fn freeze(
    req: &Request,
    Json(request): Json<FreezeProductRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Freeze product request for: {}", request.product);

    let response = freeze_product(pool, &request.product).await;

    match &response {
        FreezeProductResponse::Ok => {
            tracing::info!("Successfully froze product: {}", request.product);
        },
        FreezeProductResponse::InvalidProduct => {
            tracing::warn!("Failed to freeze product: {} does not exist", request.product);
        },
        FreezeProductResponse::AlreadyFrozen => {
            tracing::warn!("Failed to freeze product: {} is already frozen", request.product);
        },
        FreezeProductResponse::AlreadyUnfrozen => {
            // This should never happen in freeze, but we need to handle it for exhaustiveness
            tracing::error!("Unexpected AlreadyUnfrozen response in freeze handler");
        },
    }

    Ok(Json(response))
}

#[handler]
pub async fn unfreeze(
    req: &Request,
    Json(request): Json<FreezeProductRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Unfreeze product request for: {}", request.product);

    let response = unfreeze_product(pool, &request.product).await;

    match &response {
        FreezeProductResponse::Ok => {
            tracing::info!("Successfully unfroze product: {}", request.product);
        },
        FreezeProductResponse::InvalidProduct => {
            tracing::warn!("Failed to unfreeze product: {} does not exist", request.product);
        },
        FreezeProductResponse::AlreadyUnfrozen => {
            tracing::warn!("Failed to unfreeze product: {} is already unfrozen", request.product);
        },
        FreezeProductResponse::AlreadyFrozen => {
            // This should never happen in unfreeze, but we need to handle it for exhaustiveness
            tracing::error!("Unexpected AlreadyFrozen response in unfreeze handler");
        },
    }

    Ok(Json(response))
}

// Business logic for freezing a product
async fn freeze_product(
    pool: &DbPool,
    product_id: &str,
) -> FreezeProductResponse {
    // 1. Get product and verify it exists
    let product = match ProductDb::get_by_id(pool, product_id).await {
        Ok(Some(p)) => p,
        _ => return FreezeProductResponse::InvalidProduct,
    };

    // 2. Check if product is already frozen
    if product.frozen {
        return FreezeProductResponse::AlreadyFrozen;
    }

    // 3. Get current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 4. Update product to frozen state
    match ProductDb::update_frozen(pool, product_id, true, now).await {
        Ok(_) => FreezeProductResponse::Ok,
        Err(_) => FreezeProductResponse::InvalidProduct,
    }
}

// Business logic for unfreezing a product
async fn unfreeze_product(
    pool: &DbPool,
    product_id: &str,
) -> FreezeProductResponse {
    // 1. Get product and verify it exists
    let product = match ProductDb::get_by_id(pool, product_id).await {
        Ok(Some(p)) => p,
        _ => return FreezeProductResponse::InvalidProduct,
    };

    // 2. Check if product is already unfrozen
    if !product.frozen {
        return FreezeProductResponse::AlreadyUnfrozen;
    }

    // 3. Calculate how long the product was frozen
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let frozen_duration = now.saturating_sub(product.frozen_at);

    // 4. Extend started_at for all licenses with this product
    // This ensures frozen time doesn't count against users
    if let Err(e) = ProductDb::extend_started_at_for_product(pool, product_id, frozen_duration).await {
        tracing::error!("Failed to extend started_at times for product {}: {:?}", product_id, e);
        return FreezeProductResponse::InvalidProduct;
    }

    tracing::info!(
        "Extended started_at by {} seconds for all licenses with product {}",
        frozen_duration,
        product_id
    );

    // 5. Update product to unfrozen state
    match ProductDb::update_frozen(pool, product_id, false, 0).await {
        Ok(_) => FreezeProductResponse::Ok,
        Err(_) => FreezeProductResponse::InvalidProduct,
    }
}

#[handler]
pub async fn create(
    req: &Request,
    Json(request): Json<CreateProductRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Create product request for: {}", request.product);

    let response = create_product(pool, &request.product).await;

    match &response {
        CreateProductResponse::Ok => {
            tracing::info!("Successfully created product: {}", request.product);
        },
        CreateProductResponse::ProductAlreadyExists => {
            tracing::warn!("Failed to create product: {} already exists", request.product);
        },
    }

    Ok(Json(response))
}

#[handler]
pub async fn delete(
    req: &Request,
    Json(request): Json<DeleteProductFromSystemRequest>,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization
    extract_and_validate_auth(req)?;

    tracing::info!("Delete product request for: {}", request.product);

    let response = delete_product(pool, &request.product).await;

    match &response {
        DeleteProductFromSystemResponse::Ok => {
            tracing::info!("Successfully deleted product: {}", request.product);
        },
        DeleteProductFromSystemResponse::InvalidProduct => {
            tracing::warn!("Failed to delete product: {} does not exist", request.product);
        },
    }

    Ok(Json(response))
}

// Business logic for creating a product
async fn create_product(
    pool: &DbPool,
    product_id: &str,
) -> CreateProductResponse {
    // 1. Check if product already exists
    match ProductDb::exists(pool, product_id).await {
        Ok(true) => return CreateProductResponse::ProductAlreadyExists,
        Ok(false) => {},
        Err(_) => return CreateProductResponse::ProductAlreadyExists,
    }

    // 2. Create the product (not frozen by default)
    match ProductDb::create(pool, product_id, false, 0).await {
        Ok(_) => CreateProductResponse::Ok,
        Err(_) => CreateProductResponse::ProductAlreadyExists,
    }
}

// Business logic for deleting a product
async fn delete_product(
    pool: &DbPool,
    product_id: &str,
) -> DeleteProductFromSystemResponse {
    // 1. Check if product exists
    match ProductDb::exists(pool, product_id).await {
        Ok(true) => {},
        _ => return DeleteProductFromSystemResponse::InvalidProduct,
    }

    // 2. Delete the product from the products table
    // Note: This will also cascade delete from license_products due to foreign key constraints
    match ProductDb::delete(pool, product_id).await {
        Ok(_) => DeleteProductFromSystemResponse::Ok,
        Err(_) => DeleteProductFromSystemResponse::InvalidProduct,
    }
}
