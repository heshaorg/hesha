//! SHA256 operations and binding signatures for the Hesha Protocol.

use hesha_types::{HeshaResult, PhoneHash, PhoneNumber};
use sha2::{Sha256, Digest};
use rand::{RngCore, rngs::OsRng};
use base64::{Engine, engine::general_purpose};
use crate::signing::sign_message;

/// Generate a cryptographically secure random salt.
/// 
/// # Security Considerations
/// - Uses OS random number generator
/// - Returns 16 bytes of entropy
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Hash a phone number with salt using SHA256.
/// 
/// # Security Considerations
/// - Phone number is normalized before hashing
/// - Salt prevents rainbow table attacks
/// - Uses SHA256 for strong collision resistance
pub fn hash_phone_number(phone: &PhoneNumber, salt: &[u8]) -> PhoneHash {
    let mut hasher = Sha256::new();
    
    // Hash salt first, then phone number
    hasher.update(salt);
    hasher.update(phone.as_str().as_bytes());
    
    let result = hasher.finalize();
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&result);
    
    PhoneHash::from_bytes(hash_bytes)
}

/// Hash a phone number according to Hesha Protocol specification.
/// 
/// # Specification
/// - Remove '+' prefix from E.164 number
/// - Hash only the digits
/// - Return format: "sha256:hexhash"
pub fn hash_phone_number_spec(phone: &PhoneNumber) -> String {
    // Normalize: remove '+' prefix
    let normalized = phone.as_str().trim_start_matches('+');
    
    // Hash the normalized number
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    
    // Format as "sha256:hexhash"
    format!("sha256:{}", hex::encode(result))
}

/// Create binding signature according to Hesha Protocol specification v1.1.
/// 
/// # Specification
/// Ed25519-Sign(issuer_private_key, SHA256(phone_hash || user_pubkey || proxy_number || iat || "hesha-binding-v2"))
/// Returns: "sig:base64url"
pub fn create_binding_signature(
    phone_hash: &str,      // The complete "sha256:..." string
    user_pubkey: &str,     // Base64url encoded public key
    proxy_number: &str,    // Full proxy number with '+'
    iat: i64,              // Issued-at timestamp
    private_key: &hesha_types::PrivateKey,  // Issuer's Ed25519 private key
) -> HeshaResult<String> {
    // Construct canonical message according to spec v1.1
    let message = format!(
        "{}|{}|{}|{}|hesha-binding-v2",
        phone_hash, user_pubkey, proxy_number, iat
    );
    
    // Hash the message first (as per spec)
    let message_hash = sha256(message.as_bytes());
    
    // Sign the hash with issuer's private key
    let signature = sign_message(private_key, &message_hash)?;
    
    // Format as "sig:base64url"
    Ok(format!("sig:{}", general_purpose::URL_SAFE_NO_PAD.encode(signature.as_bytes())))
}

/// Verify binding signature according to protocol v1.1.
/// 
/// # Specification
/// Reconstructs the message, hashes it, and verifies the Ed25519 signature
pub fn verify_binding_signature(
    phone_hash: &str,      // The complete "sha256:..." string
    user_pubkey: &str,     // Base64url encoded public key
    proxy_number: &str,    // Full proxy number with '+'
    iat: i64,              // Issued-at timestamp
    binding_proof: &str,   // "sig:base64url" format
    issuer_pubkey: &hesha_types::PublicKey,  // Issuer's Ed25519 public key
) -> bool {
    // Check format
    if !binding_proof.starts_with("sig:") {
        return false;
    }
    
    // Extract signature
    let sig_base64 = &binding_proof[4..];
    let sig_bytes = match general_purpose::URL_SAFE_NO_PAD.decode(sig_base64) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    if sig_bytes.len() != 64 {
        return false;
    }
    
    let mut signature_array = [0u8; 64];
    signature_array.copy_from_slice(&sig_bytes);
    let signature = hesha_types::Signature::from_bytes(signature_array);
    
    // Reconstruct canonical message
    let message = format!(
        "{}|{}|{}|{}|hesha-binding-v2",
        phone_hash, user_pubkey, proxy_number, iat
    );
    
    // Hash the message
    let message_hash = sha256(message.as_bytes());
    
    // Verify signature
    crate::signing::verify_signature(issuer_pubkey, &message_hash, &signature)
}

/// Compute SHA256 hash of data.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Constant-time comparison to prevent timing attacks.
/// 
/// # Security Considerations
/// - Always processes all bytes regardless of early mismatches
/// - Uses bitwise operations to avoid branching
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use hesha_types::PhoneNumber;
    
    #[test]
    fn test_phone_hashing() {
        let phone = PhoneNumber::new("+1234567890").unwrap();
        let salt = b"test-salt";
        
        let hash1 = hash_phone_number(&phone, salt);
        let hash2 = hash_phone_number(&phone, salt);
        assert_eq!(hash1, hash2); // Should be deterministic
        
        let different_salt = b"different-salt";
        let hash3 = hash_phone_number(&phone, different_salt);
        assert_ne!(hash1, hash3); // Different salt should give different hash
    }
    
    #[test]
    fn test_phone_hashing_spec() {
        let phone = PhoneNumber::new("+1234567890").unwrap();
        let hash = hash_phone_number_spec(&phone);
        
        // Should start with "sha256:"
        assert!(hash.starts_with("sha256:"));
        
        // Should be 71 chars total (7 for prefix + 64 for hex)
        assert_eq!(hash.len(), 71);
        
        // Test known value
        assert_eq!(
            hash,
            "sha256:c775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646"
        );
    }
    
    #[test]
    fn test_binding_signature() {
        use crate::signing::generate_keypair;
        
        let issuer_key = generate_keypair().unwrap();
        let phone_hash = "sha256:c775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646";
        let user_pubkey = "MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E";
        let proxy_number = "+99012345678901";
        let iat = 1720000000i64;
        
        // Create signature
        let sig = create_binding_signature(
            phone_hash,
            user_pubkey,
            proxy_number,
            iat,
            &issuer_key.private,
        ).unwrap();
        
        // Should have correct format
        assert!(sig.starts_with("sig:"));
        
        // Verify signature
        assert!(verify_binding_signature(
            phone_hash,
            user_pubkey,
            proxy_number,
            iat,
            &sig,
            &issuer_key.public,
        ));
        
        // Wrong public key should fail
        let other_key = generate_keypair().unwrap();
        assert!(!verify_binding_signature(
            phone_hash,
            user_pubkey,
            proxy_number,
            iat,
            &sig,
            &other_key.public,
        ));
    }
    
    #[test]
    fn test_constant_time_compare() {
        let a = b"hello world";
        let b = b"hello world";
        let c = b"hello worle";
        let d = b"hello";
        
        assert!(constant_time_compare(a, b));
        assert!(!constant_time_compare(a, c));
        assert!(!constant_time_compare(a, d));
    }
    
    #[test]
    fn test_sha256() {
        let data = b"test data";
        let hash1 = sha256(data);
        let hash2 = sha256(data);
        
        // Should be deterministic
        assert_eq!(hash1, hash2);
        
        // Should be 32 bytes
        assert_eq!(hash1.len(), 32);
        
        // Different data should give different hash
        let different_data = b"different data";
        let hash3 = sha256(different_data);
        assert_ne!(hash1, hash3);
    }
}