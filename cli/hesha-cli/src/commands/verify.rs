//! Attestation verification command.

use crate::output;
use colored::*;
use hesha_core::{parse_attestation_jwt, verify_attestation};
use hesha_types::PhoneNumber;
use std::fs;

/// Execute verification.
pub async fn execute(attestation: &str, expected_phone: Option<&str>) -> anyhow::Result<()> {
    output::info("Verifying attestation...");
    
    // Load attestation (from file or direct JWT)
    let jwt = if attestation.starts_with("eyJ") {
        attestation.to_string()
    } else {
        fs::read_to_string(attestation)?
    };
    
    // Parse to check basic structure
    let _parsed = parse_attestation_jwt(&jwt)?;
    
    // Verify cryptographic proof
    match verify_attestation(&jwt).await {
        Ok(verified) => {
            output::success("Attestation is valid!");
            println!("\nDetails:");
            println!("  Issuer:       {}", verified.issuer.cyan());
            println!("  Proxy number: {}", verified.attestation.proxy_number.to_string().yellow());
            println!("  User pubkey:  {}", verified.attestation.user_pubkey.to_base64());
            println!("  Expires:      {}", verified.attestation.exp);
            
            // Check phone if provided
            if let Some(phone) = expected_phone {
                let expected = PhoneNumber::new(phone)?;
                let expected_hash = hesha_crypto::hash_phone_number_spec(&expected);
                let attestation_hash = format!("sha256:{}", verified.attestation.phone_hash.to_hex());
                
                if expected_hash == attestation_hash {
                    println!();
                    output::success("Phone number matches!");
                } else {
                    println!();
                    output::error("Phone number does not match!");
                    return Ok(()); // Not an error, just informational
                }
            }
        }
        Err(e) => {
            output::error(&format!("Attestation is invalid: {}", e));
            return Err(e.into());
        }
    }
    
    Ok(())
}