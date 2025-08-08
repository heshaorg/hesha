//! Attestation verification logic.

use crate::attestation::parse::{parse_attestation, validate_attestation};
use crate::attestation::claims::Claims;
use crate::attestation::jwt::decode_jwt;
use crate::verification::discovery::resolve_trust_domain;
use hesha_types::{HeshaResult, PublicKey, VerifiedAttestation};
use chrono::Utc;

/// Verify an attestation by discovering the issuer's public key.
/// 
/// # Security Considerations
/// - Verifies JWT signature with issuer's key
/// - Validates binding proof
/// - Checks expiry and other fields
/// - Supports trust domain resolution for subdomain deployments
pub async fn verify_attestation(jwt: &str) -> HeshaResult<VerifiedAttestation> {
    // Parse attestation to get issuer
    let attestation = parse_attestation(jwt)?;
    validate_attestation(&attestation)?;
    
    // Get the effective trust domain for verification
    let trust_domain = attestation.effective_trust_domain();
    
    // Resolve trust domain to get the actual issuer key
    let (_service_domain, issuer_key) = resolve_trust_domain(trust_domain).await?;
    
    // Verify with discovered key
    verify_attestation_with_key(jwt, &issuer_key)
}

/// Verify an attestation with a known issuer public key.
pub fn verify_attestation_with_key(
    jwt: &str,
    issuer_key: &PublicKey,
) -> HeshaResult<VerifiedAttestation> {
    // Verify JWT signature using our implementation
    let claims: Claims = decode_jwt(jwt, issuer_key)?;
    
    // Convert to attestation and validate
    let attestation = claims.to_attestation()?;
    validate_attestation(&attestation)?;
    
    // Verify binding signature with issuer's public key
    // The binding proof is stored in the claims as the full "sig:..." string
    let binding_valid = hesha_crypto::verify_binding_signature(
        &format!("sha256:{}", attestation.phone_hash.to_hex()),
        &attestation.user_pubkey.to_base64(),
        attestation.proxy_number.as_str(),
        attestation.iat.timestamp(),
        &claims.binding_proof,  // This already contains "sig:..."
        issuer_key,
    );
    
    if !binding_valid {
        return Err(hesha_types::HeshaError::InvalidAttestation(
            "Invalid binding signature".to_string()
        ));
    }
    
    Ok(VerifiedAttestation {
        attestation: attestation.clone(),
        issuer: claims.iss,
        verified_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::create::{create_attestation, create_attestation_with_trust_domain};
    use hesha_crypto::generate_keypair;
    use hesha_types::{PhoneNumber, ProxyNumber};
    
    #[test]
    fn test_verify_with_key() {
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
        
        let verified = verify_attestation_with_key(&jwt, &issuer_key.public).unwrap();
        
        assert_eq!(verified.issuer, "issuer.com");
        assert_eq!(verified.attestation.proxy_number, proxy);
        assert_eq!(verified.attestation.user_pubkey, user_key.public);
    }
    
    #[test]
    fn test_verify_with_wrong_key() {
        let issuer_key = generate_keypair().unwrap();
        let wrong_key = generate_keypair().unwrap();
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
        
        // Should fail with wrong key
        assert!(verify_attestation_with_key(&jwt, &wrong_key.public).is_err());
    }
    
    #[test]
    fn test_verify_with_trust_domain() {
        let issuer_key = generate_keypair().unwrap();
        let user_key = generate_keypair().unwrap();
        let phone = PhoneNumber::new("+1234567890").unwrap();
        let proxy = ProxyNumber::new("+23400123456789").unwrap();
        
        let jwt = create_attestation_with_trust_domain(
            "api.example.com",
            "example.com",
            &issuer_key.private,
            &phone,
            &proxy,
            &user_key.public,
        ).unwrap();
        
        // Parse and check trust domain
        let attestation = parse_attestation(&jwt).unwrap();
        assert_eq!(attestation.iss, "api.example.com");
        assert_eq!(attestation.trust_domain, Some("example.com".to_string()));
        assert_eq!(attestation.effective_trust_domain(), "example.com");
        
        // Verify with key should still work
        let verified = verify_attestation_with_key(&jwt, &issuer_key.public).unwrap();
        assert_eq!(verified.issuer, "api.example.com");
    }
}