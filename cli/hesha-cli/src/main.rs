//! Hesha Protocol CLI.

mod commands;
mod config;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hesha")]
#[command(about = "Hesha Protocol CLI - Manage proxy phone numbers and attestations")]
#[command(long_about = "
The Hesha Protocol CLI allows you to:
- Generate cryptographic keypairs for attestations
- Request proxy phone numbers from issuers
- Verify attestations and challenge responses
- Inspect attestation details

For more information about the Hesha Protocol, visit: https://github.com/hesha-protocol
")]
#[command(version)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Ed25519 keypair for attestations
    #[command(long_about = "\nGenerate a new Ed25519 keypair for use with the Hesha Protocol.\n\nThe private key is used to sign challenge responses, while the public key\nis included in attestation requests.\n\nOutput formats:\n  json   - JSON object with base64url-encoded keys (default)\n  hex    - Hexadecimal encoding\n  base64 - Base64url encoding (no padding)\n\nExamples:\n  # Generate and save to file\n  hesha keygen > keys.json\n  \n  # Generate in hex format\n  hesha keygen -f hex\n  \n  # Set as environment variable\n  export HESHA_PRIVATE_KEY=$(hesha keygen -f base64 | grep 'Private' | cut -d' ' -f3)\n")]
    Keygen {
        /// Output format (json, hex, base64)
        #[arg(short, long, default_value = "json", value_name = "FORMAT")]
        format: String,
    },
    
    /// Request attestation from an issuer
    #[command(long_about = "
Request a proxy phone number attestation from an issuer.

The issuer will verify your phone number ownership (method varies by issuer)
and provide a signed JWT attestation that binds a proxy number to your public key.

Examples:
  # Request global proxy number (+990...)
  hesha attest -i https://issuer.example.com -p +1234567890
  
  # Request US local proxy number (+1...)
  hesha attest -i https://issuer.example.com -p +1234567890 -s 1
  
  # Save attestation to file
  hesha attest -i https://issuer.example.com -p +1234567890 -o attestation.jwt
")]
    Attest {
        /// Issuer URL (e.g., https://issuer.example.com)
        #[arg(short, long, value_name = "URL")]
        issuer: String,
        
        /// Phone number to attest (E.164 format)
        #[arg(short, long, value_name = "PHONE")]
        phone: String,
        
        /// Scope - calling code for proxy number (default: 990 for global)
        #[arg(short, long, value_name = "CODE", default_value = "990")]
        scope: String,
        
        /// Private key file (or use HASHA_PRIVATE_KEY env)
        #[arg(short, long, value_name = "FILE")]
        key: Option<String>,
        
        /// Output file for attestation
        #[arg(short, long, value_name = "FILE")]
        output: Option<String>,
    },
    
    /// Verify an attestation's cryptographic validity
    #[command(long_about = "
Verify the cryptographic validity of a Hasha attestation.

This command:
1. Checks the JWT signature against the issuer's public key
2. Validates the attestation hasn't expired
3. Verifies the binding proof
4. Optionally checks if a phone number matches the attestation

Examples:
  # Verify attestation from file
  hesha verify -a attestation.jwt
  
  # Verify and check phone number
  hesha verify -a attestation.jwt -p +1234567890
  
  # Verify inline JWT
  hesha verify -a eyJ0eXAiOiJKV1Q...
")]
    Verify {
        /// Attestation file or JWT string
        #[arg(short, long, value_name = "FILE_OR_JWT")]
        attestation: String,
        
        /// Expected phone number to verify (optional)
        #[arg(short, long, value_name = "PHONE")]
        phone: Option<String>,
    },
    
    /// Display attestation details without verification
    #[command(long_about = "
Display the contents of a Hasha attestation without verification.

This command decodes and displays:
- Proxy number assigned
- Phone hash (privacy-preserved)
- Issuer domain
- User public key
- Expiration time
- Other attestation metadata

Note: This does NOT verify the attestation. Use 'hesha verify' for validation.

Examples:
  # Inspect attestation from file
  hesha inspect attestation.jwt
  
  # Inspect inline JWT
  hesha inspect eyJ0eXAiOiJKV1Q...
")]
    Inspect {
        /// Attestation file or JWT string
        #[arg(value_name = "FILE_OR_JWT")]
        attestation: String,
    },
    
    /// Display information about the Hesha Protocol
    #[command(long_about = "
Display information about the Hesha Protocol, including:
- Protocol overview and purpose
- Key concepts (proxy numbers, attestations, verification)
- Common use cases
- Links to documentation
")]
    Info,
    
    /// Setup a new Hesha issuer with interactive configuration
    #[command(name = "setup")]
    #[command(long_about = "
Initialize a new Hesha issuer node with proper configuration.

This command will:
1. Collect issuer identity information (name, domain, contact)
2. Generate Ed25519 keypairs for signing attestations
3. Configure database and server settings
4. Create all necessary directories and files
5. Generate the public key endpoint JSON

The setup process emphasizes security:
- Private keys are saved with restricted permissions
- You'll be prompted to backup your private key
- Configuration is validated before saving

Examples:
  # Interactive setup (recommended)
  hesha setup
  
  # Setup in specific directory
  hesha setup -o /path/to/issuer
  
  # Non-interactive setup with defaults
  hesha setup --non-interactive
")]
    Setup(commands::setup_issuer::SetupIssuerCmd),
    
    /// Start the Hesha issuer node
    #[command(name = "start")]
    #[command(long_about = "
Start the Hesha issuer node using a configuration created with 'hesha setup'.

Examples:
  # Start with default configuration
  hesha start
  
  # Start with named configuration
  hesha start -n myissuer
  
  # Start in background (daemon mode)
  hesha start --daemon
")]
    Start(commands::start::StartCmd),
    
    /// Stop the Hesha issuer node
    #[command(name = "stop")]
    #[command(long_about = "
Stop a running Hesha issuer node.

Examples:
  # Stop default issuer
  hesha stop
  
  # Stop named issuer
  hesha stop -n myissuer
")]
    Stop(commands::stop::StopCmd),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Set up basic logging
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    
    match cli.command {
        Commands::Keygen { format } => {
            commands::keygen::execute(&format)?;
        }
        Commands::Attest { issuer, phone, scope, key, output } => {
            commands::attest::execute(&issuer, &phone, &scope, key.as_deref(), output.as_deref()).await?;
        }
        Commands::Verify { attestation, phone } => {
            commands::verify::execute(&attestation, phone.as_deref()).await?;
        }
        Commands::Inspect { attestation } => {
            commands::inspect::execute(&attestation)?;
        }
        Commands::Info => {
            commands::info::execute()?;
        }
        Commands::Setup(cmd) => {
            cmd.execute()?;
        }
        Commands::Start(cmd) => {
            cmd.execute()?;
        }
        Commands::Stop(cmd) => {
            cmd.execute()?;
        }
    }
    
    Ok(())
}