use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::types::core::LicenseProduct;




// /auth



#[derive(Serialize, Deserialize)]
pub struct LoginRequest<'a> {
    pub license: &'a str,
    pub product: &'a str,
    pub hwid: &'a str,
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
    pub license: &'a str,
    pub product: &'a str,
    //no hwid because we can call this without caring about hwid
}

#[derive(Serialize)]
pub enum ProductResponse<'a> {
    InvalidProduct,
    InvalidLicense,
    Ok(LicenseProduct<'a>)
}


//private 

// /generator
#[derive(Deserialize)]
pub struct GeneratorRequest {
    pub products: HashMap<String, u64>, //TODO: revamp so we don't have to allocate string
}

#[derive(Serialize)]
pub enum GeneratorResponse<'a> { 
    Ok(&'a str),
    OneOrMoreInvalidProduct,
    FailedToGenerateValidLicense,
}

// /add-product

#[derive(Deserialize)]
pub struct AddProductRequest<'a> {
    pub license: &'a str,
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