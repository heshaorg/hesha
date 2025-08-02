//! HTTP client library for the Hesha Protocol.
//! 
//! This crate provides typed HTTP clients for interacting with
//! Hesha Protocol nodes:
//! 
//! - `IssuerClient`: For requesting attestations
//! - Helper types for requests and responses
//! 
//! # Security
//! 
//! - Enforces HTTPS for production use
//! - Includes timeout protection
//! - Validates response formats

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod issuer;

pub use error::{ClientError, ClientResult};
pub use issuer::{AttestationRequest, AttestationResponse, IssuerClient};