//! Attestation creation logic.

use crate::attestation::claims::Claims;
use crate::attestation::jwt::encode_jwt;
use hesha_crypto::{create_binding_signature, generate_nonce, hash_phone_number_spec};
use hesha_types::{
    Attestation, HeshaResult, PhoneNumber, PrivateKey, ProxyNumber, PublicKey,
};
use chrono::{Duration, Utc};
use uuid::Uuid;
use hex;
use base64::{Engine as _, engine::general_purpose};

/// Builder for creating attestations.
pub struct AttestationBuilder<'a> {
    issuer_domain: String,
    trust_domain: Option<String>,
    issuer_private_key: &'a PrivateKey,
    phone_number: PhoneNumber,
    proxy_number: ProxyNumber,
    user_pubkey: PublicKey,
    validity_days: i64,
}

impl<'a> AttestationBuilder<'a> {
    /// Create a new attestation builder.
    pub fn new(
        issuer_domain: String,
        issuer_private_key: &'a PrivateKey,
        phone_number: PhoneNumber,
        proxy_number: ProxyNumber,
        user_pubkey: PublicKey,
    ) -> Self {
        Self {
            issuer_domain,
            trust_domain: None,
            issuer_private_key,
            phone_number,
            proxy_number,
            user_pubkey,
            validity_days: 30, // Default 30 days
        }
    }
    
    /// Set the validity period in days.
    pub fn validity_days(mut self, days: i64) -> Self {
        self.validity_days = days;
        self
    }
    
    /// Set the trust domain (e.g., "example.com" when service runs on "api.example.com").
    pub fn trust_domain(mut self, domain: String) -> Self {
        self.trust_domain = Some(domain);
        self
    }
    
    /// Build the attestation.
    pub fn build(self) -> HeshaResult<Attestation> {
        let now = Utc::now();
        let iat = now.timestamp();
        
        // Hash phone number according to spec
        let phone_hash_str = hash_phone_number_spec(&self.phone_number);
        
        // Generate binding signature according to spec v1.1
        let binding_proof_str = create_binding_signature(
            &phone_hash_str,
            &self.user_pubkey.to_base64(),
            self.proxy_number.as_str(),
            iat,
            self.issuer_private_key,
        )?;
        
        // Parse hash from "sha256:..." format
        let hash_hex = phone_hash_str
            .strip_prefix("sha256:")
            .ok_or_else(|| hesha_types::HeshaError::CryptoError("Invalid hash format".into()))?;
        let hash_bytes: [u8; 32] = hex::decode(hash_hex)
            .map_err(|_| hesha_types::HeshaError::CryptoError("Invalid hash hex".into()))?
            .try_into()
            .map_err(|_| hesha_types::HeshaError::CryptoError("Invalid hash length".into()))?;
        
        // Store the binding proof string directly (it includes "sig:" prefix)
        // We'll need to handle this differently since BindingProof expects 32 bytes
        // For now, let's store the signature bytes in the binding proof
        let proof_base64 = binding_proof_str
            .strip_prefix("sig:")
            .ok_or_else(|| hesha_types::HeshaError::CryptoError("Invalid proof format".into()))?;
        let sig_bytes = general_purpose::URL_SAFE_NO_PAD.decode(proof_base64)
            .map_err(|_| hesha_types::HeshaError::CryptoError("Invalid proof base64".into()))?;
        
        // For compatibility with existing BindingProof type (32 bytes), 
        // we'll store first 32 bytes of the signature
        let proof_bytes: [u8; 32] = sig_bytes
            .get(..32)
            .ok_or_else(|| hesha_types::HeshaError::CryptoError("Invalid proof length".into()))?
            .try_into()
            .map_err(|_| hesha_types::HeshaError::CryptoError("Invalid proof length".into()))?;
        
        Ok(Attestation {
            proxy_number: self.proxy_number,
            phone_hash: hesha_types::PhoneHash::from_bytes(hash_bytes),
            iss: self.issuer_domain,
            trust_domain: self.trust_domain,
            exp: now + Duration::days(self.validity_days),
            iat: now,
            user_pubkey: self.user_pubkey,
            binding_proof: hesha_types::BindingProof::from_bytes(proof_bytes),
            salt: vec![],  // Not used in spec-compliant version
            jti: Uuid::new_v4().to_string(),
            nonce: generate_nonce(),  // Not included in JWT per spec
        })
    }
    
    /// Build the attestation and encode as JWT.
    pub fn build_jwt(self) -> HeshaResult<String> {
        // Store reference to issuer key before consuming self
        let issuer_key = self.issuer_private_key;
        let attestation = self.build()?;
        
        // Create binding signature for JWT
        let binding_signature = create_binding_signature(
            &format!("sha256:{}", attestation.phone_hash.to_hex()),
            &attestation.user_pubkey.to_base64(),
            attestation.proxy_number.as_str(),
            attestation.iat.timestamp(),
            issuer_key,
        )?;
        
        // Convert to JWT claims with binding signature
        let mut claims = Claims::from_attestation(&attestation);
        claims.binding_proof = binding_signature;
        
        // Encode with our Ed25519 JWT implementation
        encode_jwt(&claims, issuer_key)
    }
}

/// Create a signed JWT attestation.
/// 
/// # Security Considerations
/// - Uses Ed25519 for signing
/// - Includes all required security fields
/// - Binding signature prevents proxy number substitution and is verifiable
pub fn create_attestation(
    issuer_domain: &str,
    issuer_private_key: &PrivateKey,
    phone_number: &PhoneNumber,
    proxy_number: &ProxyNumber,
    user_pubkey: &PublicKey,
) -> HeshaResult<String> {
    AttestationBuilder::new(
        issuer_domain.to_string(),
        issuer_private_key,
        phone_number.clone(),
        proxy_number.clone(),
        user_pubkey.clone(),
    ).build_jwt()
}

/// Create a signed JWT attestation with a trust domain.
/// 
/// Use this when the issuer service runs on a subdomain (e.g., api.example.com)
/// but wants attestations to use the main domain (e.g., example.com) for trust.
pub fn create_attestation_with_trust_domain(
    issuer_domain: &str,
    trust_domain: &str,
    issuer_private_key: &PrivateKey,
    phone_number: &PhoneNumber,
    proxy_number: &ProxyNumber,
    user_pubkey: &PublicKey,
) -> HeshaResult<String> {
    AttestationBuilder::new(
        issuer_domain.to_string(),
        issuer_private_key,
        phone_number.clone(),
        proxy_number.clone(),
        user_pubkey.clone(),
    )
    .trust_domain(trust_domain.to_string())
    .build_jwt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hesha_crypto::generate_keypair;
    
    #[test]
    fn test_attestation_creation() {
        let issuer_key = generate_keypair().unwrap();
        let user_key = generate_keypair().unwrap();
        let phone = PhoneNumber::new("+1234567890").unwrap();
        let proxy = ProxyNumber::new("+23400123456789").unwrap();
        
        let attestation = AttestationBuilder::new(
            "issuer.com".to_string(),
            &issuer_key.private,
            phone.clone(),
            proxy.clone(),
            user_key.public.clone(),
        ).build().unwrap();
        
        assert_eq!(attestation.iss, "issuer.com");
        assert_eq!(attestation.trust_domain, None);
        assert_eq!(attestation.proxy_number, proxy);
        assert_eq!(attestation.user_pubkey, user_key.public);
        assert!(!attestation.is_expired());
    }
    
    #[test]
    fn test_jwt_creation() {
        let issuer_key = generate_keypair().unwrap();
        let user_key = generate_keypair().unwrap();
        let phone = PhoneNumber::new("+1234567890").unwrap();
        let proxy = ProxyNumber::new("+23400123456789").unwrap();
        
        let jwt = create_attestation(
            "issuer.com",
            &issuer_key.private,
            &phone,
            &proxy,
            &user_key.public,
        ).unwrap();
        
        // JWT should have three parts
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3);
    }
    
    #[test]
    fn test_trust_domain_creation() {
        let issuer_key = generate_keypair().unwrap();
        let user_key = generate_keypair().unwrap();
        let phone = PhoneNumber::new("+1234567890").unwrap();
        let proxy = ProxyNumber::new("+23400123456789").unwrap();
        
        let attestation = AttestationBuilder::new(
            "api.example.com".to_string(),
            &issuer_key.private,
            phone.clone(),
            proxy.clone(),
            user_key.public.clone(),
        )
        .trust_domain("example.com".to_string())
        .build().unwrap();
        
        assert_eq!(attestation.iss, "api.example.com");
        assert_eq!(attestation.trust_domain, Some("example.com".to_string()));
        assert_eq!(attestation.effective_trust_domain(), "example.com");
    }
}