//! JWT claims structure for attestations.

use hesha_types::{BindingProof, Nonce, PhoneHash, ProxyNumber, PublicKey};
use hex;
use serde::{Deserialize, Serialize};

/// JWT claims for Hesha attestations.
///
/// This matches the Hesha Protocol specification for JWT attestations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - the proxy number (standard JWT claim).
    pub sub: String,

    /// SHA256 hash of the phone number in format "sha256:hexhash".
    pub phone_hash: String,

    /// User's Ed25519 public key (base64url encoded).
    pub user_pubkey: String,

    /// Issuer domain (standard JWT claim).
    pub iss: String,

    /// Expiration time (standard JWT claim).
    pub exp: i64,

    /// Issued at time (standard JWT claim).
    pub iat: i64,

    /// JWT ID for uniqueness (standard JWT claim).
    pub jti: String,

    /// Cryptographic binding proof in format "sig:base64url".
    pub binding_proof: String,

    /// Trust domain (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_domain: Option<String>,
}

impl Claims {
    /// Convert to hesha-types Attestation.
    pub fn to_attestation(&self) -> hesha_types::HeshaResult<hesha_types::Attestation> {
        use chrono::{TimeZone, Utc};

        // Extract hash from "sha256:hexhash" format
        let hash_hex = self.phone_hash.strip_prefix("sha256:").ok_or_else(|| {
            hesha_types::HeshaError::InvalidAttestation("Invalid phone hash format".into())
        })?;

        // For now, we'll store the full signature string in binding_proof field
        // The actual signature verification happens separately
        // Extract just placeholder bytes since BindingProof expects 32 bytes
        let proof_bytes = [0u8; 32]; // Placeholder - actual verification uses the string

        Ok(hesha_types::Attestation {
            proxy_number: ProxyNumber::new(&self.sub)?,
            phone_hash: PhoneHash::from_bytes(
                hex::decode(hash_hex)
                    .map_err(|_| {
                        hesha_types::HeshaError::InvalidAttestation("Invalid phone hash hex".into())
                    })?
                    .try_into()
                    .map_err(|_| {
                        hesha_types::HeshaError::InvalidAttestation(
                            "Invalid phone hash length".into(),
                        )
                    })?,
            ),
            iss: self.iss.clone(),
            trust_domain: self.trust_domain.clone(),
            exp: Utc.timestamp_opt(self.exp, 0).single().ok_or_else(|| {
                hesha_types::HeshaError::InvalidAttestation("Invalid expiry timestamp".into())
            })?,
            iat: Utc.timestamp_opt(self.iat, 0).single().ok_or_else(|| {
                hesha_types::HeshaError::InvalidAttestation("Invalid issued timestamp".into())
            })?,
            user_pubkey: PublicKey::from_base64(&self.user_pubkey)?,
            binding_proof: BindingProof::from_bytes(proof_bytes),
            // Default values for fields not in JWT
            salt: vec![], // Not stored in JWT
            jti: self.jti.clone(),
            nonce: Nonce::new(""), // Not stored in JWT
        })
    }

    /// Create from hesha-types Attestation.
    pub fn from_attestation(attestation: &hesha_types::Attestation) -> Self {
        Claims {
            sub: attestation.proxy_number.to_string(),
            phone_hash: format!("sha256:{}", attestation.phone_hash.to_hex()),
            user_pubkey: attestation.user_pubkey.to_base64(),
            iss: attestation.iss.clone(),
            trust_domain: attestation.trust_domain.clone(),
            exp: attestation.exp.timestamp(),
            iat: attestation.iat.timestamp(),
            jti: attestation.jti.clone(),
            // For signatures, we need to reconstruct from the attestation
            // This is a temporary solution - in production, store the full signature
            binding_proof: "sig:placeholder".to_string(),
        }
    }
}
