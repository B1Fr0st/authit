use serde_derive::Serialize;
use std::sync::Arc;

/*
Owning Structure:

License |-> Sessions
        |-> License Product

Strings use Arc<str> for efficient deduplication - cheap to clone,
no memory leaks, and shared storage for repeated values.
*/

// Owned types using Arc<str> for efficient string deduplication
pub type LicenseKey = Arc<str>;
pub type HWID = Arc<str>;
pub type ProductId = Arc<str>;


#[derive(Serialize, Clone)]
pub struct License {
    pub license_key: LicenseKey,
    pub products: Vec<LicenseProduct>,
    pub hwid: HWID,
    pub sessions: Vec<Session>
}

#[derive(Serialize, Clone)]
pub struct LicenseProduct {
    pub product: ProductId,
    pub time: u64,
    pub started_at: u64
}

#[derive(Serialize, Clone)]
pub struct Product {
    pub id: ProductId,
    pub frozen: bool,
    pub frozen_at: u64
}

#[derive(Serialize, Clone)]
pub struct Session {
    pub started: u64,
    pub ended: Option<u64>
}