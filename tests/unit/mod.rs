// Unit tests are co-located with their source modules using #[cfg(test)] blocks.
//
// Run all unit tests with:
//   cargo test --lib
//
// Modules with unit tests:
//   - src/utils/crypto.rs       — AES-256-GCM encrypt/decrypt
//   - src/utils/calculator.rs   — amortisation formula
//   - src/config.rs             — Config::from_env() error handling
//   - src/errors.rs             — AppError HTTP status codes and JSON shape
//   - src/blockchain/polygon.rs — PolygonAdapter stub responses
