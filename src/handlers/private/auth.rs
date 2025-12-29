use poem::{Request, Result, Error, http::StatusCode};
use std::env;

/// Extracts and validates the Authorization header for private API endpoints
///
/// Expected format: "Bearer <api_key>"
/// The API key is validated against the API_KEY environment variable
pub fn extract_and_validate_auth(req: &Request) -> Result<()> {
    // Get the Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::from_status(StatusCode::UNAUTHORIZED));
    }

    // Extract the token
    let token = &auth_header[7..]; // Skip "Bearer "

    // Get the expected API key from environment
    let expected_key = env::var("API_KEY")
        .unwrap_or_else(|_| "default-insecure-key".to_string());

    // Validate the token
    if token != expected_key {
        return Err(Error::from_status(StatusCode::UNAUTHORIZED));
    }

    Ok(())
}

/// Authorization response for failed authentication
#[derive(serde::Serialize)]
pub struct UnauthorizedResponse {
    pub error: String,
}

impl UnauthorizedResponse {
    pub fn new() -> Self {
        Self {
            error: "Unauthorized - Valid API key required".to_string(),
        }
    }
}
