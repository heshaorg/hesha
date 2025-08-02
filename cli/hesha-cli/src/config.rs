//! Configuration and key management.

use anyhow::Context;
use dirs::home_dir;
use hesha_types::{KeyPair, PrivateKey};
use hesha_crypto::keypair_from_private;
use std::{fs, path::PathBuf};

/// Get default config directory.
pub fn config_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hesha")
}

/// Load keypair from file or environment.
pub fn load_keypair(key_path: Option<&str>) -> anyhow::Result<KeyPair> {
    // Try explicit path first
    if let Some(path) = key_path {
        let key_data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read key file: {}", path))?;
        
        return load_keypair_from_string(&key_data);
    }
    
    // Try environment variable
    if let Ok(key_data) = std::env::var("HESHA_PRIVATE_KEY") {
        return load_keypair_from_string(&key_data);
    }
    
    // Try default location
    let default_path = config_dir().join("key.json");
    if default_path.exists() {
        let key_data = fs::read_to_string(&default_path)
            .with_context(|| format!("Failed to read default key file: {:?}", default_path))?;
        
        return load_keypair_from_string(&key_data);
    }
    
    anyhow::bail!(
        "No private key found. Provide --key, set HESHA_PRIVATE_KEY, or run 'hesha keygen'"
    )
}

/// Load keypair from string (JSON or base64).
fn load_keypair_from_string(data: &str) -> anyhow::Result<KeyPair> {
    // Try JSON format first
    if data.trim().starts_with('{') {
        let json: serde_json::Value = serde_json::from_str(data)?;
        let private_key_str = json["private_key"]
            .as_str()
            .context("Missing private_key in JSON")?;
        
        let private_key = PrivateKey::from_base64(private_key_str)?;
        return Ok(keypair_from_private(&private_key)?);
    }
    
    // Try direct base64
    let private_key = PrivateKey::from_base64(data.trim())?;
    Ok(keypair_from_private(&private_key)?)
}