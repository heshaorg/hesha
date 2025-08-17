//! Issuer configuration types for the Hesha protocol.

use serde::{Deserialize, Serialize};

/// Complete issuer configuration focused on core protocol requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConfig {
    /// Issuer identity information.
    pub identity: IssuerIdentity,

    /// Port to bind the server to.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Attestation validity in days.
    #[serde(default = "default_attestation_validity")]
    pub attestation_validity_days: u32,
}

/// Issuer identity information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerIdentity {
    /// Human-readable name of the issuer (e.g., "Acme Verification Services").
    pub name: String,

    /// Trust domain where the public key is served (e.g., "issuer.example.com").
    pub trust_domain: String,

    /// Contact email for this issuer.
    pub contact_email: String,

    /// Ed25519 public key in base64url format.
    pub public_key_base64url: String,

    /// Key identifier (default: "default").
    pub key_id: String,

    /// When this issuer was created.
    pub created_at: String,
}

// Default functions for serde
fn default_port() -> u16 {
    3000
}
fn default_attestation_validity() -> u32 {
    365
}

impl IssuerConfig {
    /// Load configuration from a TOML file.
    pub fn from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: IssuerConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Get the public key discovery URL for this issuer.
    pub fn public_key_url(&self) -> String {
        format!(
            "https://{}/.well-known/hesha/pubkey.json",
            self.identity.trust_domain
        )
    }

    /// Check if this is a development configuration.
    pub fn is_development(&self) -> bool {
        self.identity.trust_domain.contains("localhost")
            || self.identity.trust_domain.contains("127.0.0.1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = IssuerConfig {
            identity: IssuerIdentity {
                name: "Test Issuer".to_string(),
                trust_domain: "issuer.example.com".to_string(),
                contact_email: "admin@example.com".to_string(),
                public_key_base64url: "test-key".to_string(),
                key_id: "default".to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            port: 3000,
            attestation_validity_days: 365,
        };

        // Test serialization
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("Test Issuer"));

        // Test deserialization
        let parsed: IssuerConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.identity.name, "Test Issuer");
    }
}
