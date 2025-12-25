//! Attestation creation and management.

pub mod claims;
pub mod create;
pub mod jwt;
pub mod parse;

pub use claims::Claims;
pub use create::{create_attestation, create_attestation_with_trust_domain, AttestationBuilder};
pub use parse::{parse_attestation, parse_attestation_jwt, validate_attestation};
