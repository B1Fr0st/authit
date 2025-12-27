use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::types::core::{HWIDRef, LicenseKeyRef, LicenseProduct, ProductIdRef};




// /auth



#[derive(Serialize, Deserialize)]
pub struct LoginRequest<'a> {
    #[serde(borrow)]
    pub license: LicenseKeyRef<'a>,
    #[serde(borrow)]
    pub product: ProductIdRef<'a>,
    #[serde(borrow)]
    pub hwid: HWIDRef<'a>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum LoginResponse {
    Ok,
    InvalidLicense,
    HWIDMismatch,
    LicenseExpired,
    LicenseFrozen
}



// /product
#[derive(Deserialize)]
pub struct ProductRequest<'a> {
    #[serde(borrow)]
    pub license: LicenseKeyRef<'a>,
    #[serde(borrow)]
    pub product: ProductIdRef<'a>,
    //no hwid because we can call this without caring about hwid
}

#[derive(Serialize)]
pub enum ProductResponse {
    InvalidProduct,
    InvalidLicense,
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
pub struct AddProductRequest<'a> {
    #[serde(borrow)]
    pub license: LicenseKeyRef<'a>,
    pub products: Vec<String>, //TODO: revamp so we don't have to allocate string
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[derive(Serialize)]
pub enum AddProductResponse {
    Ok,
    InvalidLicense,
    OneOrMoreInvalidProduct,
    LicenseExpired,
}