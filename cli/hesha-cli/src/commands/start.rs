//! Start command for running the issuer node.

use clap::Args;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use crate::output;

/// Start the Hesha issuer node.
#[derive(Debug, Args)]
pub struct StartCmd {
    /// Name of the issuer configuration to use.
    /// Defaults to 'default'
    #[arg(short, long, default_value = "default")]
    name: String,
    
    /// Run in background (daemon mode).
    #[arg(short, long)]
    daemon: bool,
}

impl StartCmd {
    pub fn execute(self) -> anyhow::Result<()> {
        // Determine config directory
        let config_dir = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".hesha")
            .join("issuer")
            .join(&self.name);
        
        // Check if configuration exists
        if !config_dir.join("config").join("issuer.toml").exists() {
            output::error(&format!("No configuration found for issuer '{}'", self.name));
            println!("\nRun 'hesha setup' first to create an issuer configuration");
            return Ok(());
        }
        
        output::info(&format!("Starting Hesha issuer node ({})", self.name));
        println!("Config: {}", config_dir.display());
        
        if self.daemon {
            // Run in background
            let pid_file = config_dir.join("issuer.pid");
            
            // Check if already running
            if pid_file.exists() {
                let pid = std::fs::read_to_string(&pid_file)?;
                output::warning(&format!("Issuer may already be running (PID: {})", pid.trim()));
                println!("Run 'hesha stop' to stop the existing instance");
                return Ok(());
            }
            
            // Find the issuer-node binary
            let issuer_bin = find_issuer_binary()?;
            
            // Start in background
            let child = Command::new(&issuer_bin)
                .env("HESHA_CONFIG_DIR", &config_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            
            // Save PID
            std::fs::write(&pid_file, child.id().to_string())?;
            
            output::success(&format!("Issuer node started in background (PID: {})", child.id()));
            println!("\nEndpoints:");
            println!("  http://localhost:3000/attest");
            println!("  http://localhost:3000/.well-known/hesha/pubkey.json");
            println!("\nRun 'hesha stop' to stop the issuer");
        } else {
            // Run in foreground using cargo
            let status = Command::new("cargo")
                .args(["run", "--bin", "issuer-node"])
                .env("HESHA_CONFIG_DIR", &config_dir)
                .status()?;
            
            if !status.success() {
                output::error("Failed to start issuer node");
                std::process::exit(1);
            }
        }
        
        Ok(())
    }
}

fn find_issuer_binary() -> anyhow::Result<PathBuf> {
    // Try release build first
    let release_path = PathBuf::from("target/release/issuer-node");
    if release_path.exists() {
        return Ok(release_path);
    }
    
    // Try debug build
    let debug_path = PathBuf::from("target/debug/issuer-node");
    if debug_path.exists() {
        return Ok(debug_path);
    }
    
    // Try system-wide installation
    if let Ok(path) = which::which("issuer-node") {
        return Ok(path);
    }
    
    anyhow::bail!("Could not find issuer-node binary. Run 'cargo build --release' first.");
}




