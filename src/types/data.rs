use serde_derive::{Deserialize, Serialize};

use crate::types::requests::LoginResponse;




pub struct Login<'a> {
    pub license: &'a str,
    pub time: u64,
    pub hwid: &'a str,
    pub response: LoginResponse
}