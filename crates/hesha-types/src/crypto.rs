//! Cryptographic key types and signatures.

use crate::error::{HeshaError, HeshaResult};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Ed25519 public key.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        PublicKey(bytes)
    }

    /// Try to create from a slice.
    pub fn try_from_slice(slice: &[u8]) -> HeshaResult<Self> {
        if slice.len() != 32 {
            return Err(HeshaError::InvalidPublicKey(format!(
                "Expected 32 bytes, got {}",
                slice.len()
            )));
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(PublicKey(bytes))
    }

    /// Get the key as bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to base64.
    pub fn to_base64(&self) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(self.0)
    }

    /// Parse from base64.
    pub fn from_base64(s: &str) -> HeshaResult<Self> {
        let bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| HeshaError::InvalidPublicKey(e.to_string()))?;
        Self::try_from_slice(&bytes)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({}...)", &self.to_base64()[..8])
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_base64().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_base64(&s).map_err(serde::de::Error::custom)
    }
}

/// Ed25519 private key.
///
/// # Security Considerations
/// - Never serialize or log private keys
/// - Always zeroize on drop (handled by ed25519-dalek)
pub struct PrivateKey([u8; 32]);

impl PrivateKey {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        PrivateKey(bytes)
    }

    /// Get the key as bytes.
    ///
    /// # Security Warning
    /// Handle with extreme care - never log or persist.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to base64 for secure storage.
    ///
    /// # Security Warning
    /// Only use for secure key storage, never log or display.
    pub fn to_base64(&self) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(self.0)
    }

    /// Parse from base64.
    pub fn from_base64(s: &str) -> HeshaResult<Self> {
        let bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| HeshaError::InvalidPrivateKey(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(HeshaError::InvalidPrivateKey(format!(
                "Expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        Ok(PrivateKey(key_bytes))
    }

    /// Create an owned copy of the private key.
    ///
    /// # Security Warning
    /// Only use when absolutely necessary (e.g., building attestations).
    pub fn to_owned(&self) -> Self {
        PrivateKey(self.0)
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateKey(**REDACTED**)")
    }
}

// Explicitly no Display, Serialize, or Deserialize for PrivateKey

/// Ed25519 key pair.
#[derive(Debug)]
pub struct KeyPair {
    /// The public key.
    pub public: PublicKey,
    /// The private key.
    pub private: PrivateKey,
}

impl KeyPair {
    /// Create a new key pair.
    pub fn new(public: PublicKey, private: PrivateKey) -> Self {
        KeyPair { public, private }
    }

    /// Create a key pair from a private key (derives the public key).
    /// This requires the hesha-crypto crate functionality.
    pub fn from_private_key(_private_key: &PrivateKey) -> HeshaResult<Self> {
        // This will be implemented by hesha-crypto
        Err(HeshaError::CryptoError(
            "KeyPair::from_private_key must be called through hesha-crypto".to_string(),
        ))
    }
}

/// Ed25519 signature.
#[derive(Clone, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Signature(bytes)
    }

    /// Try to create from a slice.
    pub fn try_from_slice(slice: &[u8]) -> HeshaResult<Self> {
        if slice.len() != 64 {
            return Err(HeshaError::InvalidSignature);
        }

        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(slice);
        Ok(Signature(bytes))
    }

    /// Get the signature as bytes.
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Convert to base64.
    pub fn to_base64(&self) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(self.0)
    }

    /// Parse from base64.
    pub fn from_base64(s: &str) -> HeshaResult<Self> {
        let bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| HeshaError::InvalidSignature)?;
        Self::try_from_slice(&bytes)
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({}...)", &self.to_base64()[..8])
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_base64().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Signature::from_base64(&s).map_err(serde::de::Error::custom)
    }
}

/// Cryptographic nonce for replay protection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nonce(String);

impl Nonce {
    /// Create a new nonce.
    pub fn new(value: impl Into<String>) -> Self {
        Nonce(value.into())
    }

    /// Get the nonce value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// HMAC-based binding proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingProof(#[serde(with = "base64_serde")] [u8; 32]);

impl BindingProof {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        BindingProof(bytes)
    }

    /// Get as bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

// Helper module for base64 serialization
mod base64_serde {
    use base64::{engine::general_purpose, Engine as _};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&general_purpose::URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(serde::de::Error::custom)?;

        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid binding proof length"));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_base64() {
        let bytes = [42u8; 32];
        let key = PublicKey::from_bytes(bytes);
        let base64 = key.to_base64();
        let decoded = PublicKey::from_base64(&base64).unwrap();
        assert_eq!(key, decoded);
    }

    #[test]
    fn test_signature_base64() {
        let bytes = [42u8; 64];
        let sig = Signature::from_bytes(bytes);
        let base64 = sig.to_base64();
        let decoded = Signature::from_base64(&base64).unwrap();
        assert_eq!(sig, decoded);
    }

    #[test]
    fn test_private_key_no_display() {
        let key = PrivateKey::from_bytes([0u8; 32]);
        let debug = format!("{:?}", key);
        assert!(debug.contains("REDACTED"));
    }
}
