pub mod login;
pub use login::*;
pub mod redeem;
pub use redeem::*;
pub mod setrole;
pub use setrole::*;
pub mod products;
pub use products::*;
use serde::{Deserialize,  Serialize};

#[derive(sqlx::Type, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Role {
    User,
    Support,
    Dev,
    Admin,
}