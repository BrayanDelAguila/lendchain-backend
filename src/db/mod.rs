use sqlx::PgPool;

pub mod admin;
pub mod loans;
pub mod payments;
pub mod users;

/// Re-export pool type for convenience.
pub type DbPool = PgPool;
