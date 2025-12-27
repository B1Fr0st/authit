use poem::{handler, http::StatusCode};



#[handler]
pub fn health() -> StatusCode {
    StatusCode::OK
}