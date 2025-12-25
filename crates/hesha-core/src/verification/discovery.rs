//! Issuer public key discovery via .well-known.

use hesha_types::{HeshaError, HeshaResult, IssuerInfo, PublicKey};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cache for issuer public keys.
#[derive(Clone)]
pub struct IssuerKeyCache {
    cache: Arc<Mutex<HashMap<String, (PublicKey, Instant)>>>,
    ttl: Duration,
}

impl IssuerKeyCache {
    /// Create a new cache with the given TTL.
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    /// Get a key from the cache if not expired.
    pub fn get(&self, domain: &str) -> Option<PublicKey> {
        let cache = self.cache.lock().ok()?;
        let (key, inserted) = cache.get(domain)?;

        if inserted.elapsed() < self.ttl {
            Some(key.clone())
        } else {
            None
        }
    }

    /// Insert a key into the cache.
    pub fn insert(&self, domain: String, key: PublicKey) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(domain, (key, Instant::now()));
        }
    }

    /// Clear the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

impl Default for IssuerKeyCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600)) // 1 hour default
    }
}

/// Discover an issuer's public key via .well-known endpoint.
///
/// # Security Considerations
/// - Always use HTTPS
/// - Validate the response format
/// - Cache results to prevent DoS
pub async fn discover_issuer_key(domain: &str) -> HeshaResult<PublicKey> {
    // Build URL - use HTTP for localhost, HTTPS for everything else
    let url = if domain.starts_with("http://") || domain.starts_with("https://") {
        return Err(HeshaError::InvalidAttestation(
            "Domain should not include protocol".to_string(),
        ));
    } else if domain.starts_with("localhost") || domain.starts_with("127.0.0.1") {
        format!("http://{}/.well-known/hesha/pubkey.json", domain)
    } else {
        format!("https://{}/.well-known/hesha/pubkey.json", domain)
    };

    // Make request with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| HeshaError::CryptoError(format!("HTTP client error: {}", e)))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| HeshaError::CryptoError(format!("Key discovery failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(HeshaError::CryptoError(format!(
            "Key discovery failed with status: {}",
            response.status()
        )));
    }

    let issuer_info: IssuerInfo = response
        .json()
        .await
        .map_err(|e| HeshaError::CryptoError(format!("Invalid issuer info JSON: {}", e)))?;

    // Validate algorithm
    if issuer_info.algorithm != "Ed25519" {
        return Err(HeshaError::CryptoError(format!(
            "Unsupported algorithm: {}",
            issuer_info.algorithm
        )));
    }

    Ok(issuer_info.public_key)
}

/// Discover an issuer's public key with caching.
pub async fn discover_issuer_key_cached(
    domain: &str,
    cache: &IssuerKeyCache,
) -> HeshaResult<PublicKey> {
    // Check cache first
    if let Some(key) = cache.get(domain) {
        return Ok(key);
    }

    // Discover and cache
    let key = discover_issuer_key(domain).await?;
    cache.insert(domain.to_string(), key.clone());

    Ok(key)
}

/// Discover issuer information including service discovery metadata.
pub async fn discover_issuer_info(domain: &str) -> HeshaResult<IssuerInfo> {
    // Build URL - use HTTP for localhost, HTTPS for everything else
    let url = if domain.starts_with("http://") || domain.starts_with("https://") {
        return Err(HeshaError::InvalidAttestation(
            "Domain should not include protocol".to_string(),
        ));
    } else if domain.starts_with("localhost") || domain.starts_with("127.0.0.1") {
        format!("http://{}/.well-known/hesha/pubkey.json", domain)
    } else {
        format!("https://{}/.well-known/hesha/pubkey.json", domain)
    };

    // Make request with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| HeshaError::CryptoError(format!("HTTP client error: {}", e)))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| HeshaError::CryptoError(format!("Key discovery failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(HeshaError::CryptoError(format!(
            "Key discovery failed with status: {}",
            response.status()
        )));
    }

    let issuer_info: IssuerInfo = response
        .json()
        .await
        .map_err(|e| HeshaError::CryptoError(format!("Invalid issuer info JSON: {}", e)))?;

    // Validate algorithm
    if issuer_info.algorithm != "Ed25519" {
        return Err(HeshaError::CryptoError(format!(
            "Unsupported algorithm: {}",
            issuer_info.algorithm
        )));
    }

    Ok(issuer_info)
}

/// Resolve trust domain to actual service domain.
/// This handles the case where a trust domain (e.g., example.com) delegates
/// to a service domain (e.g., api.example.com).
pub async fn resolve_trust_domain(trust_domain: &str) -> HeshaResult<(String, PublicKey)> {
    // First, try to discover issuer info from the trust domain
    match discover_issuer_info(trust_domain).await {
        Ok(info) => {
            // Check if there's service discovery info
            if let Some(service_info) = &info.service_info {
                // Validate the relationship
                match service_info.relationship.as_str() {
                    "subdomain" => {
                        // Extract domain from service URL
                        let service_domain = extract_domain_from_url(&service_info.service_url)?;

                        // Verify it's actually a subdomain of the trust domain
                        if !is_subdomain_of(&service_domain, trust_domain) {
                            return Err(HeshaError::CryptoError(format!(
                                "Service domain {} is not a subdomain of trust domain {}",
                                service_domain, trust_domain
                            )));
                        }

                        // Return the service domain and the public key
                        Ok((service_domain, info.public_key))
                    }
                    _ => {
                        // Unknown relationship type, use trust domain directly
                        Ok((trust_domain.to_string(), info.public_key))
                    }
                }
            } else {
                // No service info, use trust domain directly
                Ok((trust_domain.to_string(), info.public_key))
            }
        }
        Err(_) => {
            // Failed to get info from trust domain, maybe it's the service domain itself
            // Try to discover the key directly
            let key = discover_issuer_key(trust_domain).await?;
            Ok((trust_domain.to_string(), key))
        }
    }
}

/// Extract domain from a URL.
fn extract_domain_from_url(url: &str) -> HeshaResult<String> {
    if let Some(start) = url.find("://") {
        let without_protocol = &url[start + 3..];
        if let Some(end) = without_protocol.find('/') {
            Ok(without_protocol[..end].to_string())
        } else {
            Ok(without_protocol.to_string())
        }
    } else {
        Err(HeshaError::CryptoError(format!(
            "Invalid URL format: {}",
            url
        )))
    }
}

/// Check if a domain is a subdomain of another.
fn is_subdomain_of(subdomain: &str, parent: &str) -> bool {
    if subdomain == parent {
        return true;
    }

    // Remove ports if present
    let subdomain = subdomain.split(':').next().unwrap_or(subdomain);
    let parent = parent.split(':').next().unwrap_or(parent);

    // Check if subdomain ends with .parent
    subdomain.ends_with(&format!(".{}", parent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let cache = IssuerKeyCache::new(Duration::from_secs(1));
        let key = PublicKey::from_bytes([42u8; 32]);

        // Insert and retrieve
        cache.insert("example.com".to_string(), key.clone());
        assert_eq!(cache.get("example.com"), Some(key.clone()));

        // Cache miss
        assert_eq!(cache.get("other.com"), None);

        // Expiry
        std::thread::sleep(Duration::from_millis(1100));
        assert_eq!(cache.get("example.com"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = IssuerKeyCache::default();
        let key = PublicKey::from_bytes([42u8; 32]);

        cache.insert("example.com".to_string(), key);
        assert!(cache.get("example.com").is_some());

        cache.clear();
        assert!(cache.get("example.com").is_none());
    }

    #[test]
    fn test_extract_domain_from_url() {
        assert_eq!(
            extract_domain_from_url("https://api.example.com").unwrap(),
            "api.example.com"
        );
        assert_eq!(
            extract_domain_from_url("https://api.example.com/path").unwrap(),
            "api.example.com"
        );
        assert_eq!(
            extract_domain_from_url("http://localhost:3000").unwrap(),
            "localhost:3000"
        );
        assert!(extract_domain_from_url("invalid-url").is_err());
    }

    #[test]
    fn test_is_subdomain_of() {
        assert!(is_subdomain_of("api.example.com", "example.com"));
        assert!(is_subdomain_of("test.api.example.com", "example.com"));
        assert!(is_subdomain_of("example.com", "example.com"));
        assert!(!is_subdomain_of("example.com", "api.example.com"));
        assert!(!is_subdomain_of("other.com", "example.com"));

        // Test with ports
        assert!(is_subdomain_of("api.example.com:3000", "example.com"));
        assert!(is_subdomain_of("api.example.com", "example.com:80"));
    }
}
