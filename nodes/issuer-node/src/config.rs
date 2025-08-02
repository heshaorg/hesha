//! Configuration for the issuer node.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Issuer node configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Socket address to bind to.
    pub bind_address: SocketAddr,
    
    /// Issuer domain (e.g., "issuer.com").
    pub domain: String,
    
    /// Trust domain for attestations (e.g., "example.com" when service runs on "api.example.com").
    /// If not specified, defaults to the issuer domain.
    pub trust_domain: Option<String>,
    
    /// Service URL for this issuer (e.g., "https://api.example.com").
    /// Used in service discovery when trust_domain differs from domain.
    pub service_url: Option<String>,
    
    /// Path to issuer's private key file.
    pub private_key_path: Option<String>,
    
    /// Attestation validity in days.
    pub attestation_validity_days: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: ([127, 0, 0, 1], 3000).into(),
            domain: "localhost:3000".to_string(),
            trust_domain: None,
            service_url: None,
            private_key_path: None,
            attestation_validity_days: 365,
        }
    }
}