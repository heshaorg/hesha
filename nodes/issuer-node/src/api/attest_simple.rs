//! Simplified attestation endpoint that supports verification codes.
//! This is an alternative implementation for easier integration.

use crate::state::AppState;
use axum::{extract::State, Json};
use hesha_core::{create_attestation_with_trust_domain, generate_proxy_number, ProxyGenerationInput};
use hesha_crypto::{generate_keypair, generate_hex_nonce};
use hesha_types::PhoneNumber;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Simplified attestation request that supports verification codes.
#[derive(Debug, Deserialize)]
pub struct SimpleAttestationRequest {
    /// Protocol version (must be "0.1.0-alpha").
    pub version: String,
    
    /// Phone number to attest.
    pub phone: String,
    
    /// Verification code (checked against configured value in demo mode).
    pub verification_code: String,
    
    /// Scope - 1-4 digit country calling code (e.g., "1", "44", "234").
    pub scope: Option<String>,
}

/// Response containing attestation and proxy number.
#[derive(Debug, Serialize)]
pub struct SimpleAttestationResponse {
    /// The generated proxy number.
    pub proxy_number: String,
    
    /// JWT attestation that can be verified by anyone.
    pub attestation: String,
    
    /// Expiration timestamp (Unix seconds).
    pub expires_at: i64,
}

/// Handle simplified attestation request with verification code.
/// 
/// This endpoint is designed for web/mobile apps where users don't manage keys.
/// In production, integrate with your SMS verification service.
pub async fn attest_simple(
    State(state): State<AppState>,
    Json(req): Json<SimpleAttestationRequest>,
) -> Result<Json<SimpleAttestationResponse>, (axum::http::StatusCode, Json<serde_json::Value>)> {
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
    
    // Verify the code
    // TODO: In production, check against your SMS service (Twilio, etc.)
    // For demo/development, accept configured mock code
    let valid_code = std::env::var("MOCK_VERIFICATION_CODE")
        .unwrap_or_else(|_| "123456".to_string());
    
    if req.verification_code != valid_code {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_verification_code",
                "error_description": "Invalid verification code"
            }))
        ));
    }
    
    // Parse and validate phone number
    let phone_number = PhoneNumber::new(&req.phone)
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_phone_number",
                    "error_description": format!("Invalid phone number: {}", e)
                }))
            )
        })?;
    
    // Generate ephemeral keypair for this attestation
    // This key is not stored - it's just for the attestation signature
    let user_keypair = generate_keypair()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "key_generation_failed",
                    "error_description": format!("Failed to generate keys: {}", e)
                }))
            )
        })?;
    
    // Generate proxy number using new algorithm
    let nonce = generate_hex_nonce();
    let scope = req.scope.as_deref().ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "scope_required",
                "message": "Scope (country code) is required"
            }))
        )
    })?;
    let generation_input = ProxyGenerationInput {
        phone_number: req.phone.clone(),
        user_pubkey: user_keypair.public.to_base64(),
        issuer_domain: state.config.domain.clone(),
        scope: scope.to_string(),
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
    
    // Create attestation
    let attestation = if let Some(trust_domain) = &state.config.trust_domain {
        create_attestation_with_trust_domain(
            &state.config.domain,
            trust_domain,
            &state.issuer_key.private,
            &phone_number,
            &proxy_number,
            &user_keypair.public,
        )
    } else {
        hesha_core::create_attestation(
            &state.config.domain,
            &state.issuer_key.private,
            &phone_number,
            &proxy_number,
            &user_keypair.public,
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
    
    // Calculate expiration
    let expires_at = chrono::Utc::now() + chrono::Duration::days(state.config.attestation_validity_days);
    
    tracing::info!(
        "Issued simple attestation for {} -> {} (scope: {})",
        phone_number,
        proxy_number,
        scope
    );
    
    Ok(Json(SimpleAttestationResponse {
        proxy_number: proxy_number.to_string(),
        attestation,
        expires_at: expires_at.timestamp(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_simple_attestation() {
        std::env::set_var("MOCK_VERIFICATION_CODE", "123456");
        
        let req = SimpleAttestationRequest {
            version: "0.1.0-alpha".to_string(),
            phone: "+1234567890".to_string(),
            verification_code: "123456".to_string(),
            scope: None,
        };
        
        // Test request has valid format
        assert!(PhoneNumber::new(&req.phone).is_ok());
    }
}