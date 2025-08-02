//! Local proxy number generation (+{country_code}00... prefix).

use hesha_crypto::sha256;
use hesha_types::{HeshaResult, ProxyNumber};

/// Generate a local proxy number.
/// 
/// Format: +{country_code}00{digits}
pub fn generate_local_proxy(namespace: &str, index: u64, country_code: &str) -> HeshaResult<ProxyNumber> {
    // Validate country code
    if country_code.is_empty() || country_code.len() > 3 {
        return Err(hesha_types::HeshaError::InvalidProxyNumber(
            "Invalid country code".to_string()
        ));
    }
    
    if !country_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(hesha_types::HeshaError::InvalidProxyNumber(
            "Country code must be numeric".to_string()
        ));
    }
    
    // Create deterministic seed including country code
    let seed = format!("{}:{}:{}", namespace, country_code, index);
    let hash = sha256(seed.as_bytes());
    
    // Calculate how many digits we need after country code and "00"
    // Phone numbers are typically 10-15 digits total
    let prefix_len = country_code.len() + 2; // +CC00
    let remaining_digits = 12 - prefix_len; // Aim for ~12 total digits
    
    // Generate digits from hash
    let mut number = String::with_capacity(remaining_digits);
    for byte in hash.iter() {
        if number.len() >= remaining_digits {
            break;
        }
        number.push_str(&format!("{:02}", byte % 100));
    }
    
    // Trim to exact length needed
    let number = &number[..remaining_digits.min(number.len())];
    
    ProxyNumber::new_local(country_code, number)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_local_proxy() {
        let proxy1 = generate_local_proxy("issuer.com", 1, "1").unwrap();
        let proxy2 = generate_local_proxy("issuer.com", 1, "1").unwrap();
        
        // Should be deterministic
        assert_eq!(proxy1, proxy2);
        
        // Should have correct format
        assert!(!proxy1.is_global());
        assert!(proxy1.as_str().starts_with("+100"));
        
        // Different inputs should give different outputs
        let proxy3 = generate_local_proxy("issuer.com", 2, "1").unwrap();
        assert_ne!(proxy1, proxy3);
    }
    
    #[test]
    fn test_different_country_codes() {
        let us_proxy = generate_local_proxy("issuer.com", 1, "1").unwrap();
        let uk_proxy = generate_local_proxy("issuer.com", 1, "44").unwrap();
        let gh_proxy = generate_local_proxy("issuer.com", 1, "233").unwrap();
        
        assert!(us_proxy.as_str().starts_with("+100"));
        assert!(uk_proxy.as_str().starts_with("+4400"));
        assert!(gh_proxy.as_str().starts_with("+23300"));
        
        // All should be different
        assert_ne!(us_proxy, uk_proxy);
        assert_ne!(us_proxy, gh_proxy);
        assert_ne!(uk_proxy, gh_proxy);
    }
    
    #[test]
    fn test_invalid_country_codes() {
        assert!(generate_local_proxy("test", 1, "").is_err());
        assert!(generate_local_proxy("test", 1, "1234").is_err());
        assert!(generate_local_proxy("test", 1, "US").is_err());
        assert!(generate_local_proxy("test", 1, "1a").is_err());
    }
}