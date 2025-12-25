//! Cryptographic operations for the Hesha Protocol.
//!
//! This crate provides secure implementations of all cryptographic operations
//! needed by the Hesha Protocol, including:
//!
//! - Ed25519 key generation and signing
//! - SHA256 hashing for phone numbers
//! - Ed25519 signatures for binding proofs
//! - Nonce generation and validation
//! - Timing-attack resistant comparisons
//!
//! # Security Design
//!
//! - All random values use OS entropy sources
//! - Cryptographic operations use well-vetted libraries
//! - Timing attacks are mitigated where possible
//! - Private keys are handled securely

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod hashing;
pub mod nonce;
pub mod signing;

// Re-export commonly used functions
pub use hashing::{
    constant_time_compare, create_binding_signature, generate_salt, hash_phone_number,
    hash_phone_number_spec, sha256, verify_binding_signature,
};
pub use nonce::{
    generate_hex_nonce, generate_nonce, generate_timestamped_nonce, validate_timestamped_nonce,
    NonceTracker,
};
pub use signing::{
    generate_keypair, keypair_from_private, sign_challenge_response, sign_message,
    verify_challenge_response, verify_signature,
};
