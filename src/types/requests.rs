use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::types::core::LicenseProduct;




// /auth

// Struct to hold auth data extracted from headers
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    pub license: String,
    pub product: String,
    pub hwid: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum LoginResponse {
    Ok,
    InvalidLicense,
    HWIDMismatch,
    LicenseExpired,
    LicenseFrozen,
    MissingHeaders,
}



// /product

// Struct to hold product query data extracted from headers
#[derive(Debug, Clone)]
pub struct ProductHeaders {
    pub license: String,
    pub product: String,
}

#[derive(Serialize)]
pub enum ProductResponse {
    InvalidProduct,
    InvalidLicense,
    MissingHeaders,
    Ok(LicenseProduct)
}


//private 

// /generator
#[derive(Deserialize)]
pub struct GeneratorRequest {
    pub products: HashMap<String, u64>, //TODO: revamp so we don't have to allocate string
}

#[derive(Serialize)]
pub enum GeneratorResponse {
    Ok(String),
    OneOrMoreInvalidProduct,
    FailedToGenerateValidLicense,
}

// /add-product

#[derive(Deserialize)]
pub struct AddProductRequest {
    pub license: String,
    pub products: HashMap<String, u64>, // product_id -> time in seconds
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum AddProductResponse {
    Ok,
    InvalidLicense,
    OneOrMoreInvalidProduct,
}

// /delete-product

#[derive(Deserialize)]
pub struct DeleteProductRequest {
    pub license: String,
    pub products: Vec<String>, // product_ids to remove
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum DeleteProductResponse {
    Ok,
    InvalidLicense,
    OneOrMoreInvalidProduct,
}

// /freeze & /unfreeze
#[derive(Deserialize)]
pub struct FreezeProductRequest {
    pub product: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum FreezeProductResponse {
    Ok,
    InvalidProduct,
    AlreadyFrozen,
    AlreadyUnfrozen,
}

// /create (product)
#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub product: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum CreateProductResponse {
    Ok,
    ProductAlreadyExists,
}

// /delete (product)
#[derive(Deserialize)]
pub struct DeleteProductFromSystemRequest {
    pub product: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum DeleteProductFromSystemResponse {
    Ok,
    InvalidProduct,
}