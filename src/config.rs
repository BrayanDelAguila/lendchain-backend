use std::env;

/// Application configuration loaded from environment variables at startup.
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
    /// Returns `Err(String)` with a descriptive message if any required variable is missing or invalid.
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: require_env("DATABASE_URL")?,
            redis_url: require_env("REDIS_URL")?,
            jwt_secret: require_env("JWT_SECRET")?,
            wallet_encryption_key: require_env("WALLET_ENCRYPTION_KEY")?,
            polygon_rpc_url: require_env("POLYGON_RPC_URL")?,
            polygon_chain_id: require_env("POLYGON_CHAIN_ID")?
                .parse::<u64>()
                .map_err(|_| "POLYGON_CHAIN_ID must be a valid u64".to_string())?,
            polygon_contract_address: require_env("POLYGON_CONTRACT_ADDRESS")?,
            usdc_contract_address_polygon: require_env("USDC_CONTRACT_ADDRESS_POLYGON")?,
            port: require_env("BACKEND_PORT")?
                .parse::<u16>()
                .map_err(|_| "BACKEND_PORT must be a valid u16 (1–65535)".to_string())?,
            cors_origins: require_env("CORS_ORIGINS")?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            environment: require_env("ENVIRONMENT")?,
            log_level: require_env("LOG_LEVEL")?,
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

/// Returns the value of an environment variable or an `Err` with a helpful message.
fn require_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| {
        format!(
            "[Config] Required environment variable '{}' is not set. \
             Please check your .env file or environment.",
            key
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Serialise config tests to avoid env-var races between parallel test threads.
    static CONFIG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn set_all_valid_env_vars() {
        std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/test");
        std::env::set_var("REDIS_URL", "redis://localhost:6379");
        std::env::set_var("JWT_SECRET", "test_jwt_secret_32chars_minimum_ok_x");
        std::env::set_var(
            "WALLET_ENCRYPTION_KEY",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        std::env::set_var("POLYGON_RPC_URL", "https://rpc.example.com");
        std::env::set_var("POLYGON_CHAIN_ID", "80001");
        std::env::set_var("POLYGON_CONTRACT_ADDRESS", "0x0000000000000000000000000000000000000000");
        std::env::set_var(
            "USDC_CONTRACT_ADDRESS_POLYGON",
            "0x0000000000000000000000000000000000000000",
        );
        std::env::set_var("BACKEND_PORT", "8080");
        std::env::set_var("CORS_ORIGINS", "http://localhost:3000");
        std::env::set_var("ENVIRONMENT", "development");
        std::env::set_var("LOG_LEVEL", "info");
    }

    fn clear_all_env_vars() {
        for key in &[
            "DATABASE_URL", "REDIS_URL", "JWT_SECRET", "WALLET_ENCRYPTION_KEY",
            "POLYGON_RPC_URL", "POLYGON_CHAIN_ID", "POLYGON_CONTRACT_ADDRESS",
            "USDC_CONTRACT_ADDRESS_POLYGON", "BACKEND_PORT", "CORS_ORIGINS",
            "ENVIRONMENT", "LOG_LEVEL",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn test_config_missing_required_var_returns_err() {
        let _guard = CONFIG_LOCK.lock().unwrap();
        set_all_valid_env_vars();
        std::env::remove_var("DATABASE_URL");
        let result = Config::from_env();
        assert!(result.is_err(), "from_env() should return Err when DATABASE_URL is missing");
        assert!(
            result.unwrap_err().contains("DATABASE_URL"),
            "Error message should mention the missing variable"
        );
        clear_all_env_vars();
    }

    #[test]
    fn test_config_invalid_chain_id_returns_err() {
        let _guard = CONFIG_LOCK.lock().unwrap();
        set_all_valid_env_vars();
        std::env::set_var("POLYGON_CHAIN_ID", "not_a_number");
        let result = Config::from_env();
        assert!(result.is_err(), "from_env() should return Err for non-numeric POLYGON_CHAIN_ID");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("POLYGON_CHAIN_ID"),
            "Error message should mention POLYGON_CHAIN_ID, got: {}",
            msg
        );
        clear_all_env_vars();
    }

    #[test]
    fn test_config_invalid_port_returns_err() {
        let _guard = CONFIG_LOCK.lock().unwrap();
        set_all_valid_env_vars();
        std::env::set_var("BACKEND_PORT", "99999"); // u16 max is 65535
        let result = Config::from_env();
        assert!(result.is_err(), "from_env() should return Err for out-of-range BACKEND_PORT");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("BACKEND_PORT"),
            "Error message should mention BACKEND_PORT, got: {}",
            msg
        );
        clear_all_env_vars();
    }

    #[test]
    fn test_config_is_production() {
        let mut cfg = Config {
            database_url: "x".to_string(),
            redis_url: "x".to_string(),
            jwt_secret: "x".to_string(),
            wallet_encryption_key: "x".to_string(),
            polygon_rpc_url: "x".to_string(),
            polygon_chain_id: 137,
            polygon_contract_address: "x".to_string(),
            usdc_contract_address_polygon: "x".to_string(),
            port: 8080,
            cors_origins: vec![],
            environment: "production".to_string(),
            log_level: "info".to_string(),
        };
        assert!(cfg.is_production(), "is_production() should return true for 'production'");
        cfg.environment = "development".to_string();
        assert!(!cfg.is_production(), "is_production() should return false for 'development'");
        cfg.environment = "staging".to_string();
        assert!(!cfg.is_production(), "is_production() should return false for 'staging'");
    }
}
