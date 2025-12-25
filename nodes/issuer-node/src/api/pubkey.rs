//! Public key discovery endpoint.

use crate::state::AppState;
use axum::{extract::State, Json};
use chrono::Utc;
use hesha_types::attestation::ServiceDiscovery;
use hesha_types::IssuerInfo;

/// Handle .well-known public key request.
pub async fn pubkey(State(state): State<AppState>) -> Json<IssuerInfo> {
    // Build service info if trust domain differs from issuer domain
    let service_info = match (&state.config.trust_domain, &state.config.service_url) {
        (Some(trust_domain), Some(service_url)) if trust_domain != &state.config.domain => {
            Some(ServiceDiscovery {
                service_url: service_url.clone(),
                relationship: "subdomain".to_string(),
                metadata: None,
            })
        }
        _ => None,
    };

    Json(IssuerInfo {
        public_key: state.issuer_key.public.clone(),
        algorithm: "Ed25519".to_string(),
        created_at: Utc::now(), // In production, this would be the key creation time
        key_id: Some("default".to_string()),
        service_info,
    })
}
