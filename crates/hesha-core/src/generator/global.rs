//! Global proxy number generation (+990 prefix).

use hesha_crypto::sha256;
use hesha_types::{HeshaResult, ProxyNumber};

/// Generate a global proxy number (+990...).
/// 
/// Uses deterministic generation based on namespace and index.
pub fn generate_global_proxy(namespace: &str, index: u64) -> HeshaResult<ProxyNumber> {
    // Create deterministic seed
    let seed = format!("{}:{}", namespace, index);
    let hash = sha256(seed.as_bytes());
    
    // Take exactly 11 digits from hash (to stay within E.164 limit of 15 total)
    // +990 (3) + 11 digits = 14 total, well within 15 digit limit
    let mut number = String::with_capacity(11);
    
    // Generate digits from hash bytes
    for byte in hash.iter() {
        if number.len() >= 11 {
            break;
        }
        // Use single digit to have better control
        let digit = byte % 10;
        number.push_str(&format!("{}", digit));
    }
    
    // Ensure we have exactly 11 digits (pad with zeros if needed)
    while number.len() < 11 {
        number.push('0');
    }
    
    ProxyNumber::new_global(&number)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_global_proxy() {
        let proxy1 = generate_global_proxy("issuer.com", 1).unwrap();
        let proxy2 = generate_global_proxy("issuer.com", 1).unwrap();
        
        // Should be deterministic
        assert_eq!(proxy1, proxy2);
        
        // Should be global format
        assert!(proxy1.is_global());
        assert_eq!(proxy1.as_str().len(), 15); // +990 + 11 digits
        
        // Different inputs should give different outputs
        let proxy3 = generate_global_proxy("issuer.com", 2).unwrap();
        assert_ne!(proxy1, proxy3);
        
        let proxy4 = generate_global_proxy("other.com", 1).unwrap();
        assert_ne!(proxy1, proxy4);
    }
    
    #[test]
    fn test_global_proxy_format() {
        for i in 0..10 {
            let proxy = generate_global_proxy("test", i).unwrap();
            assert!(proxy.as_str().starts_with("+990"));
            
            // Verify all characters after +990 are digits
            let digits = &proxy.as_str()[4..];
            assert!(digits.chars().all(|c| c.is_ascii_digit()));
            
            // Verify total length is exactly 15
            assert_eq!(proxy.as_str().len(), 15, "Proxy {} should be 15 chars", proxy);
            
            // Print first few for verification
            if i < 3 {
                println!("Example proxy {}: {}", i, proxy);
            }
        }
    }
}