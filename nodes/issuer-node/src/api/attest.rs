//! Attestation endpoint.

use crate::state::AppState;
use axum::{extract::State, Json};
use chrono;
use hesha_core::{attestation::AttestationBuilder, generate_proxy_number, ProxyGenerationInput};
use hesha_crypto::generate_hex_nonce;
use hesha_types::{PhoneNumber, PublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Request for attestation.
#[derive(Debug, Deserialize)]
pub struct AttestationRequest {
    /// Protocol version (must be "0.1.0-alpha").
    pub version: String,
    /// Phone number (already verified by issuer's external process).
    pub phone_number: String,
    /// User's Ed25519 public key (base64url encoded).
    pub user_pubkey: String,
    /// Scope - 1-4 digit calling code (e.g., "1", "44", "234").
    pub scope: String,
    /// Optional validity period in days (defaults to issuer config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_days: Option<i64>,
}

/// Response containing attestation.
#[derive(Debug, Serialize)]
pub struct AttestationResponse {
    /// The proxy number assigned.
    pub proxy_number: String,
    /// JWT attestation.
    pub attestation: String,
    /// Expiration timestamp (Unix seconds).
    pub expires_at: i64,
}

/// Handle attestation request.
///
/// This endpoint assumes the issuer has already verified the phone number
/// through their own mechanism (SMS, carrier API, etc).
pub async fn attest(
    State(state): State<AppState>,
    Json(req): Json<AttestationRequest>,
) -> Result<Json<AttestationResponse>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    // Validate protocol version
    if req.version != "0.1.0-alpha" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_version",
                "error_description": "Only version 0.1.0-alpha is supported"
            })),
        ));
    }

    // Parse phone number
    let phone_number = PhoneNumber::new(&req.phone_number).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_phone_number",
                "error_description": format!("Invalid phone number: {}", e)
            })),
        )
    })?;

    // Parse public key
    let user_pubkey = PublicKey::from_base64(&req.user_pubkey).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_public_key",
                "error_description": format!("Invalid public key: {}", e)
            })),
        )
    })?;

    // Generate proxy number using new algorithm
    let nonce = generate_hex_nonce();
    let generation_input = ProxyGenerationInput {
        phone_number: req.phone_number.clone(),
        user_pubkey: req.user_pubkey.clone(),
        issuer_domain: state.config.domain.clone(),
        scope: req.scope.clone(),
        nonce,
    };

    let proxy_number = generate_proxy_number(&generation_input).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "proxy_generation_failed",
                "error_description": format!("Failed to generate proxy number: {}", e)
            })),
        )
    })?;

    // Determine validity days - use request value if provided, otherwise config default
    let validity_days = match req.validity_days {
        Some(days) => {
            // Enforce reasonable limits (1 day to 2 years)
            if !(1..=730).contains(&days) {
                return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_validity_days",
                        "error_description": "Validity must be between 1 and 730 days"
                    })),
                ));
            }
            days
        }
        None => state.config.attestation_validity_days,
    };

    // Create attestation using builder
    let mut builder = AttestationBuilder::new(
        state.config.domain.clone(),
        &state.issuer_key.private,
        phone_number.clone(),
        proxy_number.clone(),
        user_pubkey.clone(),
    )
    .validity_days(validity_days);

    // Add trust domain if configured
    if let Some(trust_domain) = &state.config.trust_domain {
        builder = builder.trust_domain(trust_domain.clone());
    }

    let attestation = builder.build_jwt().map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "attestation_failed",
                "error_description": format!("Failed to create attestation: {}", e)
            })),
        )
    })?;

    tracing::info!(
        "Issued attestation for {} -> {} (validity: {} days)",
        phone_number,
        proxy_number,
        validity_days
    );

    // Calculate expiration
    let expires_at = chrono::Utc::now().timestamp() + (validity_days * 24 * 3600);

    Ok(Json(AttestationResponse {
        proxy_number: proxy_number.to_string(),
        attestation,
        expires_at,
    }))
}
