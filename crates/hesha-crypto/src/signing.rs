//! Ed25519 signing operations.

use hesha_types::{HeshaResult, KeyPair, PrivateKey, PublicKey, Signature};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey};
use rand::{rngs::OsRng, RngCore};

/// Generate a new Ed25519 key pair.
/// 
/// # Security Considerations
/// - Uses OS random number generator
/// - Keys are immediately wrapped in our types for safety
pub fn generate_keypair() -> HeshaResult<KeyPair> {
    // Generate 32 random bytes for the secret key
    let mut secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_bytes);
    
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    
    let private_key = PrivateKey::from_bytes(signing_key.to_bytes());
    let public_key = PublicKey::from_bytes(verifying_key.to_bytes());
    
    Ok(KeyPair::new(public_key, private_key))
}

/// Derive a key pair from a private key.
/// 
/// # Security Considerations
/// - Derives the public key from the private key
/// - Used for loading existing keys
pub fn keypair_from_private(private_key: &PrivateKey) -> HeshaResult<KeyPair> {
    let signing_key = SigningKey::from_bytes(private_key.as_bytes());
    let verifying_key = signing_key.verifying_key();
    
    let public_key = PublicKey::from_bytes(verifying_key.to_bytes());
    
    Ok(KeyPair::new(public_key, private_key.to_owned()))
}

/// Sign a message with a private key.
/// 
/// # Security Considerations
/// - Uses deterministic signature scheme (Ed25519)
/// - Message is signed as-is without additional encoding
pub fn sign_message(private_key: &PrivateKey, message: &[u8]) -> HeshaResult<Signature> {
    let signing_key = SigningKey::from_bytes(private_key.as_bytes());
    let signature = signing_key.sign(message);
    
    Ok(Signature::from_bytes(signature.to_bytes()))
}

/// Verify a signature against a message and public key.
/// 
/// # Security Considerations
/// - Uses constant-time verification
/// - Returns false for any verification failure
pub fn verify_signature(
    public_key: &PublicKey, 
    message: &[u8], 
    signature: &Signature
) -> bool {
    let verifying_key = match VerifyingKey::from_bytes(public_key.as_bytes()) {
        Ok(key) => key,
        Err(_) => return false,
    };
    
    let sig = match ed25519_dalek::Signature::try_from(signature.as_bytes().as_slice()) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    
    verifying_key.verify(message, &sig).is_ok()
}

/// Create a signature over a formatted challenge response.
/// 
/// This creates a signature over: challenge_nonce || service_context || timestamp
pub fn sign_challenge_response(
    private_key: &PrivateKey,
    challenge_nonce: &str,
    service_context: &str,
    timestamp: &str,
) -> HeshaResult<Signature> {
    let message = format!("{}{}{}", challenge_nonce, service_context, timestamp);
    sign_message(private_key, message.as_bytes())
}

/// Verify a challenge response signature.
pub fn verify_challenge_response(
    public_key: &PublicKey,
    challenge_nonce: &str,
    service_context: &str,
    timestamp: &str,
    signature: &Signature,
) -> bool {
    let message = format!("{}{}{}", challenge_nonce, service_context, timestamp);
    verify_signature(public_key, message.as_bytes(), signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let keypair = generate_keypair().unwrap();
        
        // Keys should be different each time
        let keypair2 = generate_keypair().unwrap();
        assert_ne!(keypair.public, keypair2.public);
        assert_ne!(
            keypair.private.as_bytes(),
            keypair2.private.as_bytes()
        );
    }
    
    #[test]
    fn test_sign_verify_roundtrip() {
        let keypair = generate_keypair().unwrap();
        let message = b"test message";
        
        let signature = sign_message(&keypair.private, message).unwrap();
        assert!(verify_signature(&keypair.public, message, &signature));
        
        // Should fail with wrong message
        assert!(!verify_signature(&keypair.public, b"wrong message", &signature));
        
        // Should fail with wrong key
        let keypair2 = generate_keypair().unwrap();
        assert!(!verify_signature(&keypair2.public, message, &signature));
    }
    
    #[test]
    fn test_challenge_response_signing() {
        let keypair = generate_keypair().unwrap();
        let challenge = "nonce123";
        let context = "signal.org";
        let timestamp = "2024-01-01T00:00:00Z";
        
        let signature = sign_challenge_response(
            &keypair.private,
            challenge,
            context,
            timestamp,
        ).unwrap();
        
        assert!(verify_challenge_response(
            &keypair.public,
            challenge,
            context,
            timestamp,
            &signature,
        ));
        
        // Should fail with wrong parameters
        assert!(!verify_challenge_response(
            &keypair.public,
            "wrong-nonce",
            context,
            timestamp,
            &signature,
        ));
    }
    
    #[test]
    fn test_invalid_signatures() {
        let keypair = generate_keypair().unwrap();
        let message = b"test";
        
        // Invalid signature bytes
        let invalid_sig = Signature::from_bytes([0u8; 64]);
        assert!(!verify_signature(&keypair.public, message, &invalid_sig));
        
        // Invalid public key should return false, not panic
        let invalid_pubkey = PublicKey::from_bytes([0u8; 32]);
        let valid_sig = sign_message(&keypair.private, message).unwrap();
        assert!(!verify_signature(&invalid_pubkey, message, &valid_sig));
    }
}