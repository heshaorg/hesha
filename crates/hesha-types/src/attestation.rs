//! Attestation types and JWT claims.

use crate::{
    crypto::{BindingProof, Nonce, PublicKey, Signature},
    phone::{PhoneHash, ProxyNumber},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// JWT attestation claims.
/// 
/// This is the core data structure that proves a user owns a phone number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// The proxy number assigned to the user.
    pub proxy_number: ProxyNumber,
    
    /// SHA256 hash of the salted phone number.
    pub phone_hash: PhoneHash,
    
    /// Issuer domain (e.g., "example.com").
    pub iss: String,
    
    /// Trust domain (e.g., "example.com" when service runs on "api.example.com").
    /// If None, defaults to the issuer domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_domain: Option<String>,
    
    /// Expiration time.
    pub exp: DateTime<Utc>,
    
    /// Issued at time.
    pub iat: DateTime<Utc>,
    
    /// User's public key for challenge-response.
    pub user_pubkey: PublicKey,
    
    /// Cryptographic binding between phone_hash and proxy_number.
    pub binding_proof: BindingProof,
    
    /// Salt used for phone hashing.
    #[serde(with = "base64_serde")]
    pub salt: Vec<u8>,
    
    /// JWT ID for uniqueness.
    pub jti: String,
    
    /// Nonce for replay protection.
    pub nonce: Nonce,
}

impl Attestation {
    /// Check if the attestation has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.exp
    }
    
    /// Get the time until expiration.
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.exp - Utc::now()
    }
    
    /// Get the effective trust domain for verification.
    /// Returns the trust_domain if set, otherwise falls back to iss.
    pub fn effective_trust_domain(&self) -> &str {
        self.trust_domain.as_deref().unwrap_or(&self.iss)
    }
}

/// Challenge sent by a service for verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Random nonce from the service.
    pub nonce: Nonce,
    
    /// Service context (e.g., "signal.org").
    pub service_context: String,
    
    /// Timestamp of challenge creation.
    pub timestamp: DateTime<Utc>,
}

/// User's response to a challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// The original challenge.
    pub challenge: Challenge,
    
    /// User's signature over challenge || service_context || timestamp.
    pub signature: Signature,
    
    /// The attestation being used.
    pub attestation_id: String,
}

/// Issuer information for key discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerInfo {
    /// Issuer's public key.
    pub public_key: PublicKey,
    
    /// Algorithm (always "Ed25519" for now).
    pub algorithm: String,
    
    /// When the key was created.
    pub created_at: DateTime<Utc>,
    
    /// Optional key ID for rotation.
    pub key_id: Option<String>,
    
    /// Service discovery information for subdomain deployments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_info: Option<ServiceDiscovery>,
}

/// Service discovery information for trust domain delegation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    /// Service URL (e.g., "https://api.example.com").
    pub service_url: String,
    
    /// Trust relationship type (e.g., "subdomain", "partner").
    pub relationship: String,
    
    /// Additional metadata for verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Result of attestation verification.
#[derive(Debug, Clone)]
pub struct VerifiedAttestation {
    /// The validated attestation.
    pub attestation: Attestation,
    
    /// Issuer that signed it.
    pub issuer: String,
    
    /// When it was verified.
    pub verified_at: DateTime<Utc>,
}

// Helper module for base64 serialization
mod base64_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&general_purpose::URL_SAFE_NO_PAD.encode(bytes))
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        general_purpose::URL_SAFE_NO_PAD.decode(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::BindingProof;
    
    #[test]
    fn test_attestation_expiry() {
        let attestation = Attestation {
            proxy_number: ProxyNumber::new("+23400123456789").unwrap(),
            phone_hash: PhoneHash::from_bytes([0u8; 32]),
            iss: "example.com".to_string(),
            trust_domain: None,
            exp: Utc::now() - chrono::Duration::hours(1),
            iat: Utc::now() - chrono::Duration::hours(2),
            user_pubkey: PublicKey::from_bytes([0u8; 32]),
            binding_proof: BindingProof::from_bytes([0u8; 32]),
            salt: vec![0u8; 16],
            jti: "test-jti".to_string(),
            nonce: Nonce::new("test-nonce"),
        };
        
        assert!(attestation.is_expired());
    }
    
    #[test]
    fn test_attestation_serialization() {
        let attestation = Attestation {
            proxy_number: ProxyNumber::new("+23400123456789").unwrap(),
            phone_hash: PhoneHash::from_bytes([42u8; 32]),
            iss: "example.com".to_string(),
            trust_domain: None,
            exp: Utc::now() + chrono::Duration::hours(24),
            iat: Utc::now(),
            user_pubkey: PublicKey::from_bytes([1u8; 32]),
            binding_proof: BindingProof::from_bytes([2u8; 32]),
            salt: vec![3u8; 16],
            jti: "unique-id".to_string(),
            nonce: Nonce::new("random-nonce"),
        };
        
        let json = serde_json::to_string(&attestation).unwrap();
        let decoded: Attestation = serde_json::from_str(&json).unwrap();
        
        assert_eq!(attestation.jti, decoded.jti);
        assert_eq!(attestation.proxy_number, decoded.proxy_number);
    }
}