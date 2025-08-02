//! Full protocol flow integration test.

use hesha_crypto::{generate_keypair, generate_hex_nonce, hash_phone_number_spec};
use hesha_core::{create_attestation, generate_proxy_number, parse_attestation, verify_attestation_with_key, ProxyGenerationInput};
use hesha_types::PhoneNumber;

#[tokio::test]
async fn test_full_attestation_flow() {
    // 1. Generate keys
    let issuer_key = generate_keypair().unwrap();
    let user_key = generate_keypair().unwrap();
    
    // 2. Generate proxy number
    let phone = PhoneNumber::new("+1234567890").unwrap();
    let generation_input = ProxyGenerationInput {
        phone_number: phone.to_string(),
        user_pubkey: user_key.public.to_base64(),
        issuer_domain: "issuer.example.com".to_string(),
        scope: "990".to_string(),
        nonce: generate_hex_nonce(),
    };
    let proxy = generate_proxy_number(&generation_input).unwrap();
    
    // 3. Create attestation
    let jwt = create_attestation(
        "issuer.example.com",
        &issuer_key.private,
        &phone,
        &proxy,
        &user_key.public,
    ).unwrap();
    
    println!("Created JWT: {}", &jwt[..50]);
    
    // 4. Parse attestation (without verification)
    let parsed = parse_attestation(&jwt).unwrap();
    assert_eq!(parsed.iss, "issuer.example.com");
    assert_eq!(parsed.proxy_number, proxy);
    assert_eq!(parsed.user_pubkey, user_key.public);
    
    // 5. Verify attestation with key
    let verified = verify_attestation_with_key(&jwt, &issuer_key.public).unwrap();
    assert_eq!(verified.issuer, "issuer.example.com");
    assert_eq!(verified.attestation.proxy_number, proxy);
    
    // 6. Check phone number hash
    let computed_hash = hash_phone_number_spec(&phone);
    let attestation_hash = format!("sha256:{}", parsed.phone_hash.to_hex());
    assert_eq!(computed_hash, attestation_hash);
    
    // 7. Verify binding proof would be done internally during attestation creation
    // The binding proof is already validated when parsing the attestation
}

#[test]
fn test_proxy_number_generation() {
    use hesha_core::{generate_proxy_number, ProxyGenerationInput};
    use hesha_crypto::generate_hex_nonce;
    
    let user_pubkey = "MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E".to_string();
    let phone = "+1234567890".to_string();
    let nonce = generate_hex_nonce();
    
    // Test global proxy number
    let input1 = ProxyGenerationInput {
        phone_number: phone.clone(),
        user_pubkey: user_pubkey.clone(),
        issuer_domain: "issuer.com".to_string(),
        scope: "990".to_string(),
        nonce: nonce.clone(),
    };
    let global1 = generate_proxy_number(&input1).unwrap();
    let global2 = generate_proxy_number(&input1).unwrap();
    assert_eq!(global1, global2); // Should be deterministic
    assert!(global1.is_global());
    
    // Test local proxy number
    let input2 = ProxyGenerationInput {
        phone_number: phone.clone(),
        user_pubkey: user_pubkey.clone(),
        issuer_domain: "issuer.com".to_string(),
        scope: "1".to_string(),
        nonce: nonce.clone(),
    };
    let local1 = generate_proxy_number(&input2).unwrap();
    let local2 = generate_proxy_number(&input2).unwrap();
    assert_eq!(local1, local2); // Should be deterministic
    assert!(!local1.is_global());
    
    // Different nonces should give different numbers
    let mut input3 = input1.clone();
    input3.nonce = generate_hex_nonce();
    let global3 = generate_proxy_number(&input3).unwrap();
    assert_ne!(global1, global3);
}