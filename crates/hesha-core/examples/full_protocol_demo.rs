//! Full protocol demonstration.

use hesha_crypto::{generate_keypair, generate_hex_nonce, hash_phone_number_spec};
use hesha_core::{create_attestation, generate_proxy_number, parse_attestation, verify_attestation_with_key, ProxyGenerationInput};
use hesha_types::{PhoneNumber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Hesha Protocol Demo ===\n");
    
    // 1. Setup: Generate keys for issuer and user
    println!("1. Generating keys...");
    let issuer_key = generate_keypair()?;
    let user_key = generate_keypair()?;
    
    println!("   Issuer public key: {}", issuer_key.public.to_base64());
    println!("   User public key:   {}\n", user_key.public.to_base64());
    
    // 2. User's phone number (already verified by issuer)
    let phone = PhoneNumber::new("+1234567890")?;
    println!("2. Phone number: {}\n", phone);
    
    // 3. Generate proxy number
    println!("3. Generating proxy number...");
    let generation_input = ProxyGenerationInput {
        phone_number: phone.to_string(),
        user_pubkey: user_key.public.to_base64(),
        issuer_domain: "issuer.example.com".to_string(),
        scope: "234".to_string(),
        nonce: generate_hex_nonce(),
    };
    let proxy = generate_proxy_number(&generation_input)?;
    println!("   Proxy number: {}", proxy);
    
    // 4. Issuer creates attestation
    println!("\n4. Creating attestation...");
    let jwt = create_attestation(
        "issuer.example.com",
        &issuer_key.private,
        &phone,
        &proxy,
        &user_key.public,
    )?;
    
    println!("   JWT (first 100 chars): {}...\n", &jwt[..100.min(jwt.len())]);
    
    // 5. Parse attestation (without verification)
    println!("5. Parsing attestation...");
    let parsed = parse_attestation(&jwt)?;
    
    println!("   Issuer:       {}", parsed.iss);
    println!("   Proxy:        {}", parsed.proxy_number);
    println!("   Expires:      {}", parsed.exp);
    println!("   Phone hash:   {}\n", parsed.phone_hash.to_hex());
    
    // 6. Verify attestation
    println!("6. Verifying attestation...");
    let verified = verify_attestation_with_key(&jwt, &issuer_key.public)?;
    
    println!("   ✓ Signature valid!");
    println!("   Verified at: {}\n", verified.verified_at);
    
    // 7. Verify phone number matches
    println!("7. Checking phone number...");
    let computed_hash = hash_phone_number_spec(&phone);
    let attestation_hash = format!("sha256:{}", parsed.phone_hash.to_hex());
    
    if computed_hash == attestation_hash {
        println!("   ✓ Phone number matches!");
    } else {
        println!("   ✗ Phone number does not match!");
        println!("   Expected: {}", computed_hash);
        println!("   Got:      {}", attestation_hash);
    }
    
    println!("\n=== Demo Complete ===");
    
    Ok(())
}