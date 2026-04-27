use actix_web::web;

pub mod admin;
pub mod loans;
pub mod payments;
pub mod users;

/// Register all v1 routes under /api/v1.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(users::configure)
            .configure(loans::configure)
            .configure(payments::configure)
            .configure(admin::configure),
    );
}
