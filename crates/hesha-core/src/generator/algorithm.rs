//! Proxy number generation algorithm according to Hesha Protocol specification v1.0

use crate::generator::ProxyGenerationInput;
use hesha_crypto::sha256;
use hesha_types::{HeshaError, HeshaResult, ProxyNumber};
use regex::Regex;

/// Generate proxy number following the exact specification algorithm.
pub fn generate(input: &ProxyGenerationInput) -> HeshaResult<ProxyNumber> {
    // Step 1: Validate inputs
    validate_inputs(input)?;

    // Step 2: Construct input string
    let input_string = format!(
        "{}|{}|{}|{}|{}",
        input.phone_number, input.user_pubkey, input.issuer_domain, input.scope, input.nonce
    );

    // Step 3: Generate hash
    let hash_bytes = sha256(input_string.as_bytes());

    // Step 4: Extract digits
    let digits = extract_digits(&hash_bytes);

    // Step 5: Format as proxy number
    format_proxy_number(&input.scope, &digits)
}

/// Validate all inputs according to specification.
fn validate_inputs(input: &ProxyGenerationInput) -> HeshaResult<()> {
    // Validate phone number (E.164)
    let phone_regex = Regex::new(r"^\+[1-9]\d{6,14}$").unwrap();
    if !phone_regex.is_match(&input.phone_number) {
        return Err(HeshaError::InvalidPhoneNumber(format!(
            "Invalid phone number format: {}",
            input.phone_number
        )));
    }

    // Validate scope (1-4 digit calling code)
    let scope_regex = Regex::new(r"^[1-9]\d{0,3}$").unwrap();
    if !scope_regex.is_match(&input.scope) {
        return Err(HeshaError::CryptoError(format!(
            "Invalid scope: {}",
            input.scope
        )));
    }

    // Validate nonce (32 hex chars, lowercase)
    let nonce_regex = Regex::new(r"^[a-f0-9]{32}$").unwrap();
    if !nonce_regex.is_match(&input.nonce) {
        return Err(HeshaError::InvalidNonce);
    }

    // Note: Public key validation is done when parsing from base64url
    // Domain validation is minimal - just ensure non-empty
    if input.issuer_domain.is_empty() {
        return Err(HeshaError::CryptoError(
            "Issuer domain cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Extract digits from hash bytes using the specification algorithm.
fn extract_digits(hash_bytes: &[u8]) -> String {
    let mut digits = String::with_capacity(20);
    let hex_string = hex::encode(hash_bytes);

    // Process all hex characters (64 chars from SHA256)
    for hex_char in hex_string.chars() {
        let hex_digit = hex_char.to_digit(16).unwrap();
        let decimal_digit = hex_digit % 10;
        digits.push_str(&decimal_digit.to_string());

        if digits.len() >= 20 {
            break;
        }
    }

    digits
}

/// Format the proxy number according to specification.
fn format_proxy_number(scope: &str, digits: &str) -> HeshaResult<ProxyNumber> {
    let cc_len = scope.len();

    // Calculate digits after "00" pattern
    let digits_after_00 = (15 - cc_len - 3).clamp(8, 10);

    // Take the required number of digits
    let proxy_digits = &digits[..digits_after_00];

    // Format: +{scope}00{digits}
    let proxy_number = format!("+{}00{}", scope, proxy_digits);

    // Create ProxyNumber (it will validate E.164 compliance)
    ProxyNumber::new(&proxy_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_example() {
        // Test with example from specification
        let input = ProxyGenerationInput {
            phone_number: "+1234567890".to_string(),
            user_pubkey: "MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E".to_string(),
            issuer_domain: "example.com".to_string(),
            scope: "234".to_string(),
            nonce: "a1b2c3d4e5f67890a1b2c3d4e5f67890".to_string(),
        };

        let proxy = generate(&input).unwrap();
        println!("Generated proxy: {}", proxy.as_str());
        assert!(proxy.as_str().starts_with("+23400"));
        assert_eq!(proxy.as_str().len(), 15); // +234 + 00 + 9 digits = 15
    }

    #[test]
    fn test_different_scope_lengths() {
        let base_input = ProxyGenerationInput {
            phone_number: "+1234567890".to_string(),
            user_pubkey: "MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E".to_string(),
            issuer_domain: "example.com".to_string(),
            scope: "1".to_string(),
            nonce: "a1b2c3d4e5f67890a1b2c3d4e5f67890".to_string(),
        };

        // Test 1-digit scope
        let proxy1 = generate(&base_input).unwrap();
        println!("Generated proxy for scope '1': {}", proxy1.as_str());
        assert!(proxy1.as_str().starts_with("+100"));
        assert_eq!(proxy1.as_str().len(), 14); // +1 + 00 + 10 digits = 14

        // Test 2-digit scope
        let mut input2 = base_input.clone();
        input2.scope = "44".to_string();
        let proxy2 = generate(&input2).unwrap();
        assert!(proxy2.as_str().starts_with("+4400"));
        assert_eq!(proxy2.as_str().len(), 15); // +44 + 00 + 10 digits = 15

        // Test 3-digit scope
        let mut input3 = base_input.clone();
        input3.scope = "233".to_string();
        let proxy3 = generate(&input3).unwrap();
        assert!(proxy3.as_str().starts_with("+23300"));
        assert_eq!(proxy3.as_str().len(), 15); // +233 + 00 + 9 digits = 15

        // Test 4-digit scope
        let mut input4 = base_input.clone();
        input4.scope = "1264".to_string();
        let proxy4 = generate(&input4).unwrap();
        assert!(proxy4.as_str().starts_with("+126400"));
        assert_eq!(proxy4.as_str().len(), 15); // +1264 + 00 + 8 digits = 15
    }

    #[test]
    fn test_input_validation() {
        let mut input = ProxyGenerationInput {
            phone_number: "+1234567890".to_string(),
            user_pubkey: "MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E".to_string(),
            issuer_domain: "example.com".to_string(),
            scope: "1".to_string(),
            nonce: "a1b2c3d4e5f67890a1b2c3d4e5f67890".to_string(),
        };

        // Invalid phone number
        input.phone_number = "1234567890".to_string(); // Missing +
        assert!(generate(&input).is_err());

        // Invalid scope
        input.phone_number = "+1234567890".to_string();
        input.scope = "0".to_string(); // Can't start with 0
        assert!(generate(&input).is_err());

        // Invalid nonce
        input.scope = "1".to_string();
        input.nonce = "INVALID".to_string();
        assert!(generate(&input).is_err());
    }
}
