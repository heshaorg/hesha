//! Attestation parsing and validation.

use crate::attestation::claims::Claims;
use crate::attestation::jwt::decode_jwt_unverified;
use hesha_types::{Attestation, HeshaError, HeshaResult};

/// Parse a JWT attestation without verifying the signature.
///
/// # Security Warning
/// This does NOT verify the signature. Use `verify_attestation` for that.
pub fn parse_attestation(jwt: &str) -> HeshaResult<Attestation> {
    // Decode without validation using our JWT implementation
    let claims: Claims = decode_jwt_unverified(jwt)?;
    claims.to_attestation()
}

/// Validate attestation fields without signature verification.
///
/// Checks:
/// - All required fields are present
/// - Proxy number format is valid
/// - Timestamps are reasonable
pub fn validate_attestation(attestation: &Attestation) -> HeshaResult<()> {
    // Check expiry
    if attestation.is_expired() {
        return Err(HeshaError::AttestationExpired(attestation.exp));
    }

    // Check issued time is not in future
    let now = chrono::Utc::now();
    if attestation.iat > now {
        return Err(HeshaError::InvalidAttestation(
            "Attestation issued in the future".to_string(),
        ));
    }

    // Check validity period is reasonable (not more than 1 year)
    let max_validity = chrono::Duration::days(365);
    if attestation.exp - attestation.iat > max_validity {
        return Err(HeshaError::InvalidAttestation(
            "Attestation validity period too long".to_string(),
        ));
    }

    // Validate issuer domain format
    if attestation.iss.is_empty() {
        return Err(HeshaError::InvalidAttestation(
            "Empty issuer domain".to_string(),
        ));
    }

    // Allow localhost for testing, otherwise require a proper domain
    if !attestation.iss.starts_with("localhost") && !attestation.iss.contains('.') {
        return Err(HeshaError::InvalidAttestation(
            "Invalid issuer domain".to_string(),
        ));
    }

    Ok(())
}

/// Parse a JWT attestation without verifying the signature.
///
/// Alias for `parse_attestation` for clarity.
pub fn parse_attestation_jwt(jwt: &str) -> HeshaResult<Attestation> {
    parse_attestation(jwt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::create::create_attestation;
    use hesha_crypto::generate_keypair;
    use hesha_types::{PhoneNumber, ProxyNumber};

    #[test]
    fn test_parse_attestation() {
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
        )
        .unwrap();

        let attestation = parse_attestation(&jwt).unwrap();

        assert_eq!(attestation.iss, "issuer.com");
        assert_eq!(attestation.proxy_number, proxy);
        assert_eq!(attestation.user_pubkey, user_key.public);
    }

    #[test]
    fn test_validate_attestation() {
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
        )
        .unwrap();

        let attestation = parse_attestation(&jwt).unwrap();
        assert!(validate_attestation(&attestation).is_ok());
    }

    #[test]
    fn test_expired_attestation() {
        use chrono::{Duration, Utc};
        use hesha_types::{Attestation, BindingProof, Nonce, PhoneHash};

        let user_key = generate_keypair().unwrap();
        let expired_attestation = Attestation {
            proxy_number: ProxyNumber::new("+23400123456789").unwrap(),
            phone_hash: PhoneHash::from_bytes([0u8; 32]),
            iss: "issuer.com".to_string(),
            trust_domain: None,
            exp: Utc::now() - Duration::hours(1), // Expired
            iat: Utc::now() - Duration::days(31),
            user_pubkey: user_key.public,
            binding_proof: BindingProof::from_bytes([0u8; 32]),
            salt: vec![0u8; 16],
            jti: "test".to_string(),
            nonce: Nonce::new("test"),
        };

        assert!(validate_attestation(&expired_attestation).is_err());
    }
}
