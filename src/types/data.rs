use crate::types::core::{HWID, LicenseKey};
use crate::types::requests::LoginResponse;
use serde_derive::Serialize;

#[derive(Serialize, Clone)]
pub struct Login {
    pub license: LicenseKey,
    pub time: u64,
    pub hwid: HWID,
    pub response: LoginResponse
}