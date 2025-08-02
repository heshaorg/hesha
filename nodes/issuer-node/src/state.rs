//! Application state management.

use crate::config::Config;
use hesha_types::KeyPair;
use std::sync::Arc;

/// Application state.
#[derive(Clone)]
pub struct AppState {
    /// Configuration.
    pub config: Config,
    /// Issuer's key pair.
    pub issuer_key: Arc<KeyPair>,
}

impl AppState {
    /// Create new app state.
    pub fn new(config: Config, issuer_key: KeyPair) -> Self {
        Self {
            config,
            issuer_key: Arc::new(issuer_key),
        }
    }
}