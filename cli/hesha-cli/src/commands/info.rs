//! Info command to display protocol information.

use colored::*;

/// Execute info command.
pub fn execute() -> anyhow::Result<()> {
    println!("{}", "Hesha Protocol Information".bold().cyan());
    println!("{}", "=========================".cyan());
    println!();
    
    println!("{}", "Overview:".bold());
    println!("The Hesha Protocol provides privacy-preserving proxy phone numbers that can be");
    println!("verified cryptographically without revealing your real phone number.");
    println!();
    
    println!("{}", "Key Concepts:".bold());
    println!("• {} - Anonymized phone numbers that preserve privacy", "Proxy Numbers".yellow());
    println!("  - Global format: +990XXXXXXXXXXX");
    println!("  - Local format: +<country_code>00XXXXXXXX");
    println!();
    
    println!("• {} - Signed JWTs proving phone ownership", "Attestations".yellow());
    println!("  - Issued by trusted issuer nodes");
    println!("  - Bind proxy numbers to user public keys");
    println!("  - Include cryptographic binding proofs");
    println!();
    
    println!("• {} - Challenge-response authentication", "Verification".yellow());
    println!("  - Services challenge users to prove key ownership");
    println!("  - Users sign challenges with their private keys");
    println!("  - No phone number revealed during verification");
    println!();
    
    println!("{}", "Common Commands:".bold());
    println!("• {} - Generate a new Ed25519 keypair", "hesha keygen".green());
    println!("• {} - Request proxy number attestation", "hesha attest".green());
    println!("• {} - Verify an attestation", "hesha verify".green());
    println!("• {} - View attestation details", "hesha inspect".green());
    println!();
    
    println!("{}", "Example Workflow:".bold());
    println!("1. Generate keypair: {}", "hesha keygen -f json > keys.json".dimmed());
    println!("2. Request attestation: {}", "hesha attest -i https://issuer.com -p +1234567890 -k keys.json".dimmed());
    println!("3. Use proxy number in apps that support Hesha Protocol");
    println!();
    
    println!("{}", "Environment Variables:".bold());
    println!("• {} - Default private key for attestations", "HESHA_PRIVATE_KEY".yellow());
    println!("• {} - Enable debug logging", "RUST_LOG=debug".yellow());
    println!();
    
    println!("{}", "Learn More:".bold());
    println!("• Documentation: {}", "https://github.com/hesha-protocol/hesha".blue());
    println!("• Specification: {}", "https://github.com/hesha-protocol/hesha/docs".blue());
    println!();
    
    Ok(())
}