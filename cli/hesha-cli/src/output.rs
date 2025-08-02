//! Output formatting utilities.

use colored::*;

/// Print a success message.
pub fn success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Print an error message.
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

/// Print a warning message.
pub fn warning(message: &str) {
    eprintln!("{} {}", "!".yellow(), message);
}

/// Print an info message.
pub fn info(message: &str) {
    println!("{} {}", "ℹ".cyan(), message);
}