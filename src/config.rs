use std::env;

/// Application configuration loaded from environment variables at startup.
/// If any required variable is missing, the process will panic with a clear message.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub wallet_encryption_key: String,
    pub polygon_rpc_url: String,
    pub polygon_chain_id: u64,
    pub polygon_contract_address: String,
    pub usdc_contract_address_polygon: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub environment: String,
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables.
    /// Panics with a descriptive message if any required variable is missing or invalid.
    pub fn from_env() -> Self {
        Self {
            database_url: require_env("DATABASE_URL"),
            redis_url: require_env("REDIS_URL"),
            jwt_secret: require_env("JWT_SECRET"),
            wallet_encryption_key: require_env("WALLET_ENCRYPTION_KEY"),
            polygon_rpc_url: require_env("POLYGON_RPC_URL"),
            polygon_chain_id: require_env("POLYGON_CHAIN_ID")
                .parse::<u64>()
                .expect("POLYGON_CHAIN_ID must be a valid u64"),
            polygon_contract_address: require_env("POLYGON_CONTRACT_ADDRESS"),
            usdc_contract_address_polygon: require_env("USDC_CONTRACT_ADDRESS_POLYGON"),
            port: require_env("BACKEND_PORT")
                .parse::<u16>()
                .expect("BACKEND_PORT must be a valid u16 (1–65535)"),
            cors_origins: require_env("CORS_ORIGINS")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            environment: require_env("ENVIRONMENT"),
            log_level: require_env("LOG_LEVEL"),
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

/// Returns the value of an environment variable or panics with a helpful message.
fn require_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!(
            "[Config] Required environment variable '{}' is not set. \
             Please check your .env file or environment.",
            key
        )
    })
}
