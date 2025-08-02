//! Attestation endpoint.

use crate::state::AppState;
use axum::{extract::State, Json};
use hesha_core::{create_attestation, create_attestation_with_trust_domain, generate_proxy_number, ProxyGenerationInput};
use hesha_crypto::generate_hex_nonce;
use hesha_types::{PhoneNumber, PublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono;

/// Request for attestation.
#[derive(Debug, Deserialize)]
pub struct AttestationRequest {
    /// Protocol version (must be "0.1.0-alpha").
    pub version: String,
    /// Phone number (already verified by issuer's external process).
    pub phone_number: String,
    /// User's Ed25519 public key (base64url encoded).
    pub user_pubkey: String,
    /// Scope - 1-4 digit calling code (e.g., "1", "44", "990").
    pub scope: String,
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
            }))
        ));
    }
    
    // Parse phone number
    let phone_number = PhoneNumber::new(&req.phone_number)
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_phone_number",
                    "error_description": format!("Invalid phone number: {}", e)
                }))
            )
        })?;
    
    // Parse public key
    let user_pubkey = PublicKey::from_base64(&req.user_pubkey)
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_public_key",
                    "error_description": format!("Invalid public key: {}", e)
                }))
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
    
    let proxy_number = generate_proxy_number(&generation_input)
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "proxy_generation_failed",
                    "error_description": format!("Failed to generate proxy number: {}", e)
                }))
            )
        })?;
    
    // Create attestation with trust domain if configured
    let attestation = if let Some(trust_domain) = &state.config.trust_domain {
        create_attestation_with_trust_domain(
            &state.config.domain,
            trust_domain,
            &state.issuer_key.private,
            &phone_number,
            &proxy_number,
            &user_pubkey,
        )
    } else {
        create_attestation(
            &state.config.domain,
            &state.issuer_key.private,
            &phone_number,
            &proxy_number,
            &user_pubkey,
        )
    }
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "attestation_failed",
                "error_description": format!("Failed to create attestation: {}", e)
            }))
        )
    })?;
    
    tracing::info!(
        "Issued attestation for {} -> {}",
        phone_number,
        proxy_number
    );
    
    // For now, use a default expiration of 30 days (standard attestation validity)
    let expires_at = chrono::Utc::now().timestamp() + (30 * 24 * 3600);
    
    Ok(Json(AttestationResponse {
        proxy_number: proxy_number.to_string(),
        attestation,
        expires_at,
    }))
}