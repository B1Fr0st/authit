use serde_derive::{Deserialize, Serialize};


#[derive(Serialize)]
pub struct License<'a> {
    pub products: Vec<LicenseProduct<'a>>,
    pub hwid: &'a str,
    pub sessions: Vec<Session>
}   

#[derive(Serialize)]
pub struct LicenseProduct<'a> {
    pub product: &'a Product<'a>,
    pub time: u64,
    pub started_at: u64
}

#[derive(Serialize, Deserialize)]
pub struct Product<'a> {
    pub id: &'a str,
    pub frozen: bool,
    pub frozen_at: u64
}

#[derive(Serialize, Deserialize)]
pub struct Session { 
    pub started: u64,
    pub ended: Option<u64>
}