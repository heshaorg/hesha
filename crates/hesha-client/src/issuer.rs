//! Client for interacting with issuer nodes.

use crate::error::{ClientError, ClientResult};
use hesha_types::{PhoneNumber, PublicKey};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request for attestation.
#[derive(Debug, Serialize)]
pub struct AttestationRequest {
    /// Protocol version.
    pub version: String,
    /// Phone number (already verified by issuer).
    pub phone_number: String,
    /// User's public key.
    pub user_pubkey: String,
    /// Scope for proxy number generation (e.g., "1" for US, "44" for UK, "234" for Nigeria).
    pub scope: String,
    /// Optional validity period in days (defaults to issuer config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_days: Option<i64>,
}

/// Response containing attestation.
#[derive(Debug, Deserialize)]
pub struct AttestationResponse {
    /// JWT attestation.
    pub attestation: String,
    /// The proxy number assigned.
    pub proxy_number: String,
}

/// Client for issuer node operations.
#[derive(Clone)]
pub struct IssuerClient {
    client: Client,
    base_url: Url,
}

impl IssuerClient {
    /// Create a new issuer client.
    pub fn new(base_url: &str) -> ClientResult<Self> {
        let base_url = Url::parse(base_url).map_err(|e| ClientError::InvalidUrl(e.to_string()))?;

        // Ensure HTTPS for security
        if base_url.scheme() != "https"
            && !base_url.host_str().unwrap_or("").starts_with("localhost")
        {
            return Err(ClientError::InvalidUrl(
                "Issuer URL must use HTTPS".to_string(),
            ));
        }

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self { client, base_url })
    }

    /// Create a client for testing (allows HTTP).
    #[cfg(any(test, debug_assertions))]
    pub fn new_insecure(base_url: &str) -> ClientResult<Self> {
        let base_url = Url::parse(base_url).map_err(|e| ClientError::InvalidUrl(e.to_string()))?;

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self { client, base_url })
    }

    /// Request attestation for a verified phone number.
    ///
    /// Note: scope is now required by the protocol. Use the phone's country code
    /// or specify a different scope for the proxy number.
    pub async fn request_attestation(
        &self,
        phone_number: &PhoneNumber,
        user_pubkey: &PublicKey,
        scope: &str,
        validity_days: Option<i64>,
    ) -> ClientResult<AttestationResponse> {
        let url = self
            .base_url
            .join("attest")
            .map_err(|e| ClientError::InvalidUrl(e.to_string()))?;

        let request = AttestationRequest {
            version: "0.1.0-alpha".to_string(),
            phone_number: phone_number.to_string(),
            user_pubkey: user_pubkey.to_base64(),
            scope: scope.to_string(),
            validity_days,
        };

        let response = self.client.post(url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ClientError::ServerError { status, message });
        }

        response
            .json()
            .await
            .map_err(|e| ClientError::InvalidResponse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        // HTTPS required
        assert!(IssuerClient::new("https://issuer.com").is_ok());
        assert!(IssuerClient::new("http://issuer.com").is_err());

        // Localhost allowed
        assert!(IssuerClient::new("http://localhost:8080").is_ok());

        // Invalid URL
        assert!(IssuerClient::new("not a url").is_err());
    }
}
