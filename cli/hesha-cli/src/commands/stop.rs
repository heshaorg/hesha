//! Stop command for terminating the issuer node.

use crate::output;
use clap::Args;

/// Stop the Hesha issuer node.
#[derive(Debug, Args)]
pub struct StopCmd {
    /// Name of the issuer configuration.
    /// Defaults to 'default'
    #[arg(short, long, default_value = "default")]
    name: String,
}

impl StopCmd {
    pub fn execute(self) -> anyhow::Result<()> {
        // Determine config directory
        let config_dir = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".hesha")
            .join("issuer")
            .join(&self.name);

        let pid_file = config_dir.join("issuer.pid");

        if !pid_file.exists() {
            output::info("No running issuer node found");
            return Ok(());
        }

        // Read PID
        let pid_str = std::fs::read_to_string(&pid_file)?;
        let pid: u32 = pid_str.trim().parse()?;

        output::info(&format!("Stopping issuer node (PID: {})", pid));

        // Send terminate signal
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            match signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {
                    // Remove PID file
                    std::fs::remove_file(&pid_file)?;
                    output::success("Issuer node stopped");
                }
                Err(e) => {
                    // Process might have already exited
                    if e == nix::errno::Errno::ESRCH {
                        std::fs::remove_file(&pid_file)?;
                        output::warning("Issuer node was not running (cleaned up PID file)");
                    } else {
                        output::error(&format!("Failed to stop issuer: {}", e));
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            output::error("Stop command is not supported on this platform yet");
            println!("Please stop the issuer node manually");
        }

        Ok(())
    }
}
