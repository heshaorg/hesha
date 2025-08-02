//! Enhanced configuration loader with environment support.

use crate::config::Config;
use serde::Deserialize;
use std::{env, fs, net::SocketAddr, path::Path};

#[derive(Debug, Deserialize)]
struct FileConfig {
    server: ServerConfig,
    #[serde(default)]
    security: SecurityConfig,
    #[serde(default)]
    keys: KeysConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    bind_address: SocketAddr,
    domain: String,
    #[serde(default = "default_environment")]
    environment: String,
}

#[derive(Debug, Deserialize, Default)]
struct SecurityConfig {
    #[serde(default = "default_rate_limit")]
    rate_limit_per_minute: u32,
    #[serde(default = "default_max_attestations")]
    max_attestations_per_phone: u32,
    #[serde(default = "default_validity_days")]
    attestation_validity_days: i64,
}

#[derive(Debug, Deserialize, Default)]
struct KeysConfig {
    private_key_path: Option<String>,
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_rate_limit() -> u32 {
    60
}

fn default_max_attestations() -> u32 {
    5
}

fn default_validity_days() -> i64 {
    365
}

impl Config {
    /// Load configuration from file and environment.
    pub fn load() -> anyhow::Result<Self> {
        // Check for config file path
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "issuer-config.toml".to_string());
        
        // Load from file if exists
        let config = if Path::new(&config_path).exists() {
            let contents = fs::read_to_string(&config_path)?;
            let file_config: FileConfig = toml::from_str(&contents)?;
            
            Config {
                bind_address: file_config.server.bind_address,
                domain: file_config.server.domain,
                private_key_path: file_config.keys.private_key_path,
                attestation_validity_days: file_config.security.attestation_validity_days,
            }
        } else {
            // Fall back to environment/defaults
            Config {
                bind_address: env::var("BIND_ADDRESS")
                    .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
                    .parse()?,
                domain: env::var("ISSUER_DOMAIN")
                    .unwrap_or_else(|_| "localhost:3000".to_string()),
                private_key_path: env::var("PRIVATE_KEY_PATH").ok(),
                attestation_validity_days: env::var("ATTESTATION_VALIDITY_DAYS")
                    .unwrap_or_else(|_| "365".to_string())
                    .parse()?,
            }
        };
        
        Ok(config)
    }
    
    /// Check if running in production.
    pub fn is_production(&self) -> bool {
        !self.domain.contains("localhost") && !self.domain.contains("127.0.0.1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.bind_address.port(), 3000);
        assert_eq!(config.domain, "localhost:3000");
    }
}