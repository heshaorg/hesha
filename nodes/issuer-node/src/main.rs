//! Hesha Protocol issuer node.

mod api;
mod config;
mod state;

use crate::config::Config;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use hesha_crypto::generate_keypair;
use std::fs;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "issuer_node=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load configuration (from environment or defaults)
    let config = Config::default();
    
    // Generate or load issuer key
    let issuer_key = if let Some(key_path) = &config.private_key_path {
        // Load from file
        let key_data = fs::read_to_string(key_path)?;
        let private_key = hesha_types::PrivateKey::from_base64(key_data.trim())?;
        hesha_crypto::keypair_from_private(&private_key)?
    } else {
        // Generate new
        generate_keypair()?
    };
    tracing::info!(
        "Issuer public key: {}",
        issuer_key.public.to_base64()
    );
    
    // Create app state
    let state = AppState::new(config.clone(), issuer_key);
    
    // Build router
    let app = Router::new()
        .route("/attest", post(api::attest::attest))
        .route("/attest/simple", post(api::attest_simple::attest_simple))
        .route("/.well-known/hesha/pubkey.json", get(api::pubkey::pubkey))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    // Start server
    let addr = config.bind_address;
    tracing::info!("Issuer node listening on {}", addr);
    tracing::info!("Endpoints:");
    tracing::info!("  POST   /attest                     - Issue attestation (with user pubkey)");
    tracing::info!("  POST   /attest/simple              - Issue attestation (with verification code)");
    tracing::info!("  GET    /.well-known/hesha/pubkey.json - Public key discovery");
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hesha_crypto::generate_keypair;
    use hesha_types::PhoneNumber;
    use tower::util::ServiceExt;
    
    #[tokio::test]
    async fn test_attestation_issuance() {
        // Start test server
        let config = Config::default();
        let issuer_key = generate_keypair().unwrap();
        let state = AppState::new(config.clone(), issuer_key);
        
        let app = Router::new()
            .route("/attest", post(api::attest::attest))
            .route("/.well-known/hesha/pubkey.json", get(api::pubkey::pubkey))
            .with_state(state);
        
        // Create test client
        let client = tower::ServiceBuilder::new()
            .service(app);
        
        // Test attestation request
        let user_key = generate_keypair().unwrap();
        let phone = PhoneNumber::new("+1234567890").unwrap();
        
        let request = serde_json::json!({
            "version": "0.1.0-alpha",
            "phone_number": phone.to_string(),
            "user_pubkey": user_key.public.to_base64(),
            "scope": "990",
        });
        
        let response = client
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/attest")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&request).unwrap()))
                    .unwrap()
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert!(result["attestation"].is_string());
        assert!(result["proxy_number"].is_string());
        assert!(result["expires_at"].is_number());
    }
}