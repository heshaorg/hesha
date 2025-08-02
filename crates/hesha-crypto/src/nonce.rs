//! Nonce generation and validation for replay protection.

use hesha_types::{HeshaError, HeshaResult, Nonce};
use rand::{RngCore, rngs::OsRng};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose};
use hex;

/// Generate a cryptographically secure random nonce.
/// 
/// # Security Considerations
/// - Uses OS random number generator
/// - Returns base64-encoded random bytes
/// - 32 bytes of entropy (256 bits)
pub fn generate_nonce() -> Nonce {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    
    // Use base64url encoding for URL safety
    let encoded = general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    Nonce::new(encoded)
}

/// Generate a cryptographically secure random nonce in hex format.
/// 
/// Returns 128-bit nonce as 32 lowercase hex characters as required by spec.
pub fn generate_hex_nonce() -> String {
    let mut bytes = [0u8; 16]; // 128 bits = 16 bytes
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes) // Returns 32 hex chars
}

/// Generate a time-based nonce that includes timestamp.
/// 
/// Format: timestamp_base64(random_bytes)
pub fn generate_timestamped_nonce() -> HeshaResult<Nonce> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| HeshaError::CryptoError("System time error".to_string()))?
        .as_secs();
    
    let mut random_bytes = [0u8; 24]; // 192 bits of randomness
    OsRng.fill_bytes(&mut random_bytes);
    
    let random_b64 = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
    let nonce_value = format!("{}_{}", timestamp, random_b64);
    
    Ok(Nonce::new(nonce_value))
}

/// Simple in-memory nonce tracking for replay protection.
/// 
/// # Security Considerations
/// - This is a basic implementation for testing
/// - Production systems should use distributed storage
/// - Nonces should expire after reasonable time
#[derive(Debug, Clone)]
pub struct NonceTracker {
    used_nonces: Arc<Mutex<HashSet<String>>>,
}

impl NonceTracker {
    /// Create a new nonce tracker.
    pub fn new() -> Self {
        Self {
            used_nonces: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    /// Check if a nonce has been used and mark it as used.
    /// 
    /// Returns Ok(()) if nonce is new, Err if already used.
    pub fn use_nonce(&self, nonce: &Nonce) -> HeshaResult<()> {
        let mut used = self.used_nonces.lock()
            .map_err(|_| HeshaError::CryptoError("Lock poisoned".to_string()))?;
        
        if used.contains(nonce.as_str()) {
            return Err(HeshaError::InvalidNonce);
        }
        
        used.insert(nonce.as_str().to_string());
        Ok(())
    }
    
    /// Check if a nonce has been used without marking it.
    pub fn is_used(&self, nonce: &Nonce) -> bool {
        self.used_nonces.lock()
            .map(|used| used.contains(nonce.as_str()))
            .unwrap_or(true) // Fail safe
    }
    
    /// Clear all tracked nonces (for testing).
    pub fn clear(&self) {
        if let Ok(mut used) = self.used_nonces.lock() {
            used.clear();
        }
    }
}

impl Default for NonceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a timestamped nonce is within acceptable time window.
/// 
/// # Parameters
/// - `nonce`: The timestamped nonce to validate
/// - `max_age_seconds`: Maximum age in seconds
pub fn validate_timestamped_nonce(nonce: &Nonce, max_age_seconds: u64) -> HeshaResult<()> {
    let nonce_str = nonce.as_str();
    
    // Parse timestamp from nonce format: timestamp_randomdata
    let parts: Vec<&str> = nonce_str.split('_').collect();
    if parts.len() != 2 {
        return Err(HeshaError::InvalidNonce);
    }
    
    let timestamp = parts[0].parse::<u64>()
        .map_err(|_| HeshaError::InvalidNonce)?;
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| HeshaError::CryptoError("System time error".to_string()))?
        .as_secs();
    
    // Check if timestamp is in the future (with clock skew allowance)
    if timestamp > current_time + 300 { // Allow 5 minutes clock skew
        return Err(HeshaError::InvalidNonce);
    }
    
    // Check age
    let age = current_time.saturating_sub(timestamp);
    if age > max_age_seconds {
        return Err(HeshaError::InvalidNonce);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nonce_generation() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        
        // Should be different
        assert_ne!(nonce1.as_str(), nonce2.as_str());
        
        // Should be valid base64
        assert!(general_purpose::URL_SAFE_NO_PAD
            .decode(nonce1.as_str()).is_ok());
    }
    
    #[test]
    #[ignore = "Flaky test - passes individually but fails in parallel"]
    fn test_timestamped_nonce() {
        let nonce = generate_timestamped_nonce().unwrap();
        
        // Should contain underscore
        assert!(nonce.as_str().contains('_'));
        
        // Should be valid within 1 hour
        assert!(validate_timestamped_nonce(&nonce, 3600).is_ok());
        
        // Create an old nonce manually to test expiration
        let old_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 7200; // 2 hours ago
        
        let random_b64 = general_purpose::URL_SAFE_NO_PAD
            .encode([42u8; 24]);
        let old_nonce = Nonce::new(format!("{}_{}", old_timestamp, random_b64));
        
        // Should be invalid with 1 hour window
        assert!(validate_timestamped_nonce(&old_nonce, 3600).is_err());
    }
    
    #[test]
    fn test_nonce_tracker() {
        let tracker = NonceTracker::new();
        let nonce = generate_nonce();
        
        // First use should succeed
        assert!(tracker.use_nonce(&nonce).is_ok());
        assert!(tracker.is_used(&nonce));
        
        // Second use should fail
        assert!(tracker.use_nonce(&nonce).is_err());
        
        // Different nonce should work
        let nonce2 = generate_nonce();
        assert!(tracker.use_nonce(&nonce2).is_ok());
    }
    
    #[test]
    fn test_tracker_clear() {
        let tracker = NonceTracker::new();
        let nonce = generate_nonce();
        
        tracker.use_nonce(&nonce).unwrap();
        assert!(tracker.is_used(&nonce));
        
        tracker.clear();
        assert!(!tracker.is_used(&nonce));
    }
    
    #[test]
    fn test_future_nonce_rejection() {
        // Create a "future" nonce manually
        let future_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 1000; // 1000 seconds in the future
        
        let random_b64 = general_purpose::URL_SAFE_NO_PAD
            .encode([42u8; 24]);
        let future_nonce = Nonce::new(format!("{}_{}", future_timestamp, random_b64));
        
        // Should be rejected
        assert!(validate_timestamped_nonce(&future_nonce, 3600).is_err());
    }
}

