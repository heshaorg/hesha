//! Simple JWT implementation for Ed25519.

use base64::{Engine as _, engine::general_purpose};
use hesha_crypto::{sign_message, verify_signature};
use hesha_types::{HeshaError, HeshaResult, PrivateKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};

/// JWT header for Ed25519.
#[derive(Debug, Serialize, Deserialize)]
struct Header {
    alg: String,
    typ: String,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        }
    }
}

/// Encode a JWT with Ed25519 signature.
pub fn encode_jwt<T: Serialize>(
    claims: &T,
    private_key: &PrivateKey,
) -> HeshaResult<String> {
    // Create header
    let header = Header::default();
    let header_json = serde_json::to_string(&header)
        .map_err(|e| HeshaError::SerializationError(e.to_string()))?;
    let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(header_json);
    
    // Encode claims
    let claims_json = serde_json::to_string(claims)
        .map_err(|e| HeshaError::SerializationError(e.to_string()))?;
    let claims_b64 = general_purpose::URL_SAFE_NO_PAD.encode(claims_json);
    
    // Create signature input
    let message = format!("{}.{}", header_b64, claims_b64);
    
    // Sign with Ed25519
    let signature = sign_message(private_key, message.as_bytes())?;
    let signature_b64 = general_purpose::URL_SAFE_NO_PAD.encode(signature.as_bytes());
    
    // Combine into JWT
    Ok(format!("{}.{}.{}", header_b64, claims_b64, signature_b64))
}

/// Decode and verify a JWT with Ed25519.
pub fn decode_jwt<T: for<'de> Deserialize<'de>>(
    jwt: &str,
    public_key: &PublicKey,
) -> HeshaResult<T> {
    // Split JWT
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err(HeshaError::InvalidAttestation("Invalid JWT format".to_string()));
    }
    
    // Verify header
    let header_json = general_purpose::URL_SAFE_NO_PAD.decode(parts[0])
        .map_err(|_| HeshaError::InvalidAttestation("Invalid header encoding".to_string()))?;
    let header: Header = serde_json::from_slice(&header_json)
        .map_err(|e| HeshaError::InvalidAttestation(format!("Invalid header: {}", e)))?;
    
    if header.alg != "EdDSA" {
        return Err(HeshaError::InvalidAttestation(
            format!("Unsupported algorithm: {}", header.alg)
        ));
    }
    
    // Decode signature
    let signature_bytes = general_purpose::URL_SAFE_NO_PAD.decode(parts[2])
        .map_err(|_| HeshaError::InvalidAttestation("Invalid signature encoding".to_string()))?;
    let signature = Signature::try_from_slice(&signature_bytes)?;
    
    // Verify signature
    let message = format!("{}.{}", parts[0], parts[1]);
    if !verify_signature(public_key, message.as_bytes(), &signature) {
        return Err(HeshaError::InvalidSignature);
    }
    
    // Decode and return claims
    let claims_json = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
        .map_err(|_| HeshaError::InvalidAttestation("Invalid claims encoding".to_string()))?;
    serde_json::from_slice(&claims_json)
        .map_err(|e| HeshaError::InvalidAttestation(format!("Invalid claims: {}", e)))
}

/// Decode JWT without verification (for parsing).
pub fn decode_jwt_unverified<T: for<'de> Deserialize<'de>>(
    jwt: &str,
) -> HeshaResult<T> {
    // Split JWT
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err(HeshaError::InvalidAttestation("Invalid JWT format".to_string()));
    }
    
    // Decode claims
    let claims_json = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
        .map_err(|_| HeshaError::InvalidAttestation("Invalid claims encoding".to_string()))?;
    serde_json::from_slice(&claims_json)
        .map_err(|e| HeshaError::InvalidAttestation(format!("Invalid claims: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hesha_crypto::generate_keypair;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestClaims {
        sub: String,
        exp: i64,
    }
    
    #[test]
    fn test_jwt_roundtrip() {
        let keypair = generate_keypair().unwrap();
        let claims = TestClaims {
            sub: "test".to_string(),
            exp: 1234567890,
        };
        
        // Encode
        let jwt = encode_jwt(&claims, &keypair.private).unwrap();
        assert!(jwt.contains('.'));
        
        // Decode with verification
        let decoded: TestClaims = decode_jwt(&jwt, &keypair.public).unwrap();
        assert_eq!(decoded, claims);
        
        // Should fail with wrong key
        let wrong_key = generate_keypair().unwrap();
        assert!(decode_jwt::<TestClaims>(&jwt, &wrong_key.public).is_err());
    }
    
    #[test]
    fn test_unverified_decode() {
        let keypair = generate_keypair().unwrap();
        let claims = TestClaims {
            sub: "test".to_string(),
            exp: 1234567890,
        };
        
        let jwt = encode_jwt(&claims, &keypair.private).unwrap();
        let decoded: TestClaims = decode_jwt_unverified(&jwt).unwrap();
        assert_eq!(decoded, claims);
    }
}