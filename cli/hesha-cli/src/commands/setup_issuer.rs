//! Setup command for initializing a new issuer.

use clap::Args;
use hesha_core::{IssuerSetupBuilder};
use dialoguer::{theme::ColorfulTheme, Input, Confirm};
use crate::output;
use std::path::PathBuf;

/// Setup a new Hesha issuer with interactive configuration.
#[derive(Debug, Args)]
pub struct SetupIssuerCmd {
    /// Output directory for configuration and keys.
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,
    
    /// Skip interactive prompts and use defaults.
    #[arg(long)]
    non_interactive: bool,
}

impl SetupIssuerCmd {
    pub fn execute(self) -> anyhow::Result<()> {
        // Fun ASCII art banner
        println!("\n{}", "═".repeat(60));
        println!(r#"
    ╦ ╦┌─┐┌─┐┬ ┬┌─┐  ╦┌─┐┌─┐┬ ┬┌─┐┬─┐
    ╠═╣├┤ └─┐├─┤├─┤  ║└─┐└─┐│ │├┤ ├┬┘
    ╩ ╩└─┘└─┘┴ ┴┴ ┴  ╩└─┘└─┘└─┘└─┘┴└─
    
    🚀 Let's create your issuer identity! 🚀
        "#);
        println!("{}\n", "═".repeat(60));
        
        output::info("Welcome to Hesha Protocol Setup");
        println!("Setting up your issuer node for the first time.\n");
        
        let theme = ColorfulTheme::default();
        
        // Collect configuration interactively
        let name = if self.non_interactive {
            "Test Issuer".to_string()
        } else {
            Input::with_theme(&theme)
                .with_prompt("Enter your issuer name (e.g., 'Acme Verification Services')")
                .validate_with(|input: &String| {
                    if input.trim().is_empty() {
                        Err("Issuer name cannot be empty")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?
        };
        
        let trust_domain = if self.non_interactive {
            "issuer.example.com".to_string()
        } else {
            println!("\nYour trust domain is where verifiers will find your public key.");
            println!("This MUST be a domain you control and can serve HTTPS content from.");
            Input::with_theme(&theme)
                .with_prompt("Enter your trust domain (e.g., 'issuer.example.com')")
                .validate_with(|input: &String| {
                    if input.contains("://") {
                        Err("Please enter domain only, without protocol")
                    } else if !input.contains('.') {
                        Err("Invalid domain format")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?
        };
        
        let contact_email = if self.non_interactive {
            "admin@example.com".to_string()
        } else {
            Input::with_theme(&theme)
                .with_prompt("Enter contact email for this issuer")
                .validate_with(|input: &String| {
                    if !input.contains('@') {
                        Err("Invalid email format")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?
        };
        
        
        
        // Build the configuration
        println!("\n{}", "─".repeat(60));
        println!("🔑 Generating Ed25519 keypair for signing attestations...");
        println!("{}", "─".repeat(60));
        
        let setup = IssuerSetupBuilder::new()
            .name(&name)
            .trust_domain(&trust_domain)
            .contact_email(&contact_email)
            .build()?;
        
        // Review configuration
        if !self.non_interactive {
            println!("\n📋 Review Configuration");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Issuer Name: {}", setup.config.identity.name);
            println!("Trust Domain: {}", setup.config.identity.trust_domain);
            println!("Public Key URL: {}", setup.public_key_url());
            println!("Contact Email: {}", setup.config.identity.contact_email);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let confirm = Confirm::with_theme(&theme)
                .with_prompt("Is this configuration correct?")
                .interact()?;
            
            if !confirm {
                println!("Setup cancelled.");
                return Ok(());
            }
        }
        
        // Save configuration
        output::info("Saving configuration and keys...");
        setup.save(&self.output_dir)?;
        
        output::success(&format!("✅ Configuration saved to {}/config/issuer.toml", self.output_dir.display()));
        output::success(&format!("✅ Keys saved to {}/keys/", self.output_dir.display()));
        
        // Display public key for easy copying
        println!("\n{}", "─".repeat(60));
        println!("📋 Your Public Key (copy this!):");
        println!("{}", "─".repeat(60));
        println!("\n{}\n", setup.config.identity.public_key_base64url);
        println!("{}", "─".repeat(60));
        
        // Show critical backup information with ASCII art
        println!("\n{}", "═".repeat(60));
        println!(r#"
    ⚠️  🔐 CRITICAL: BACKUP YOUR PRIVATE KEY! 🔐 ⚠️
    
    ┌─────────────────────────────────────────┐
    │   Your private key is stored at:       │
    │   {}/keys/private.key                   │
    └─────────────────────────────────────────┘
        "#, self.output_dir.display());
        println!("{}", "═".repeat(60));
        
        println!("\n📋 Backup Checklist:");
        println!("  □ Copy private key to encrypted storage");
        println!("  □ Store backups in multiple locations");
        println!("  □ Never commit to version control");
        println!("  □ Consider hardware security module (HSM)");
        
        // Success message with celebration
        println!("\n{}", "─".repeat(60));
        println!(r#"
    ✨ 🎉 Setup Complete! 🎉 ✨
    
    Your issuer identity has been created!
        "#);
        println!("{}", "─".repeat(60));
        
        // Next steps with nice formatting
        println!("\n🚀 Next Steps:\n");
        
        println!("1️⃣  Deploy your public key:");
        println!("    📍 URL: https://{}/.well-known/hesha/pubkey.json", setup.config.identity.trust_domain);
        println!("    📄 File: {}/config/public-key-endpoint.json\n", self.output_dir.display());
        
        println!("2️⃣  Configure DNS:");
        println!("    🌐 Point {} → your server IP\n", setup.config.identity.trust_domain);
        
        println!("3️⃣  Start your issuer node:");
        println!("    💻 cargo run --bin hesha-issuer-node\n");
        
        println!("4️⃣  Test everything:");
        println!("    🧪 hesha verify --help\n");
        
        println!("{}", "═".repeat(60));
        println!("Happy issuing! 🚀");
        println!("{}\n", "═".repeat(60));
        
        Ok(())
    }
}