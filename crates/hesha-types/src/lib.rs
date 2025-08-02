//! Core types for the Hesha Protocol.
//! 
//! This crate provides the fundamental data structures used throughout the
//! Hesha Protocol implementation. It has no dependencies on other Hasha crates
//! and focuses on type safety and validation.
//! 
//! # Security Considerations
//! 
//! - Phone numbers are validated but should be hashed before storage
//! - Private keys are never serializable and redacted in debug output
//! - All types enforce validation rules at construction time

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod attestation;
pub mod crypto;
pub mod error;
pub mod phone;

// Re-export commonly used types
pub use attestation::{
    Attestation, Challenge, ChallengeResponse, IssuerInfo, VerifiedAttestation,
};
pub use crypto::{
    BindingProof, KeyPair, Nonce, PrivateKey, PublicKey, Signature,
};
pub use error::{HeshaError, HeshaResult};
pub use phone::{PhoneHash, PhoneNumber, ProxyNumber};

#[cfg(test)]
mod tests;