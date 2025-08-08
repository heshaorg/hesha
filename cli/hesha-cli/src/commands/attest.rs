//! Attestation request command.

use crate::{config, output};
use colored::*;
use hesha_client::IssuerClient;
use hesha_types::PhoneNumber;
use std::fs;

/// Execute attestation request.
pub async fn execute(
    issuer: &str,
    phone: &str,
    scope: &str,
    key_path: Option<&str>,
    output: Option<&str>,
    validity_days: Option<i64>,
) -> anyhow::Result<()> {
    output::info("Requesting attestation...");
    
    // Load private key
    let keypair = config::load_keypair(key_path)?;
    
    // Parse phone number
    let phone_number = PhoneNumber::new(phone)?;
    
    // Create client
    let client = IssuerClient::new(issuer)?;
    
    // Request attestation with specified scope and optional validity
    let response = client
        .request_attestation(&phone_number, &keypair.public, scope, validity_days)
        .await?;
    
    output::success("Attestation received!");
    println!("Proxy number: {}", response.proxy_number.yellow());
    
    // Save attestation
    if let Some(output_path) = output {
        fs::write(output_path, &response.attestation)?;
        println!("Saved to: {}", output_path.cyan());
    } else {
        println!("\nAttestation JWT:");
        println!("{}", response.attestation);
    }
    
    Ok(())
}