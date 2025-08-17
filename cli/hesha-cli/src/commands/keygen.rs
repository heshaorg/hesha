//! Key generation command.

use crate::output;
use hesha_crypto::generate_keypair;
use serde_json::json;

/// Execute keygen command.
pub fn execute(format: &str) -> anyhow::Result<()> {
    let keypair = generate_keypair()?;

    match format {
        "json" => {
            let output = json!({
                "private_key": keypair.private.to_base64(),
                "public_key": keypair.public.to_base64(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "hex" => {
            println!("Private key: {}", hex::encode(keypair.private.as_bytes()));
            println!("Public key:  {}", hex::encode(keypair.public.as_bytes()));
        }
        "base64" => {
            println!("Private key: {}", keypair.private.to_base64());
            println!("Public key:  {}", keypair.public.to_base64());
        }
        _ => {
            anyhow::bail!("Unknown format: {}. Use json, hex, or base64", format);
        }
    }

    eprintln!();
    output::success("Keys generated successfully!");
    output::warning("Store your private key securely.");

    Ok(())
}
