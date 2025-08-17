//! Phone number types and validation.

use crate::error::{HeshaError, HeshaResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated real phone number.
///
/// # Security Considerations
/// - Phone numbers are validated but never stored in logs
/// - Always hash before persisting
#[derive(Clone, PartialEq, Eq)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Create a new phone number with validation.
    ///
    /// # Validation Rules
    /// - Must start with '+'
    /// - Must contain only digits after '+'
    /// - Must be between 7 and 15 digits (excluding '+')
    pub fn new(number: impl Into<String>) -> HeshaResult<Self> {
        let number = number.into();

        // Remove any whitespace
        let cleaned = number
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        // Validate format
        if !cleaned.starts_with('+') {
            return Err(HeshaError::InvalidPhoneNumber(
                "Phone number must start with '+'".to_string(),
            ));
        }

        let digits = &cleaned[1..];
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(HeshaError::InvalidPhoneNumber(
                "Phone number must contain only digits after '+'".to_string(),
            ));
        }

        let digit_count = digits.len();
        if !(7..=15).contains(&digit_count) {
            return Err(HeshaError::InvalidPhoneNumber(format!(
                "Phone number must have 7-15 digits, found {}",
                digit_count
            )));
        }

        Ok(PhoneNumber(cleaned))
    }

    /// Get the phone number as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the country code (1-3 digits after '+').
    pub fn country_code(&self) -> &str {
        let digits = &self.0[1..];

        // Try to identify country code length based on known patterns
        if digits.starts_with('1') || digits.starts_with('7') {
            &self.0[1..2] // US/Canada (+1) or Russia (+7)
        } else if digits.len() >= 2 {
            // Most country codes are 2-3 digits
            &self.0[1..3.min(self.0.len())]
        } else {
            &self.0[1..2]
        }
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact phone number in debug output for privacy
        write!(f, "PhoneNumber(+***)")
    }
}

// Secure serialization - be careful about logging
impl Serialize for PhoneNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PhoneNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PhoneNumber::new(s).map_err(serde::de::Error::custom)
    }
}

/// A validated proxy phone number.
///
/// # Format
/// - +{country_code}00XXXXXXXXX (country code followed by 00 and digits)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyNumber(String);

impl ProxyNumber {
    /// Create a new proxy number with validation.
    pub fn new(number: impl Into<String>) -> HeshaResult<Self> {
        let number = number.into();

        // Remove any whitespace
        let cleaned = number
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        // Validate format
        if !cleaned.starts_with('+') {
            return Err(HeshaError::InvalidProxyNumber(
                "Proxy number must start with '+'".to_string(),
            ));
        }

        let digits = &cleaned[1..];
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(HeshaError::InvalidProxyNumber(
                "Proxy number must contain only digits after '+'".to_string(),
            ));
        }

        // Check if it's a valid proxy format
        if cleaned.starts_with("+990") {
            return Err(HeshaError::InvalidProxyNumber(
                "Global proxy numbers (+990) are no longer supported".to_string(),
            ));
        } else {
            // Local proxy number - must have '00' after country code
            let has_double_zero = digits.contains("00");

            if !has_double_zero {
                return Err(HeshaError::InvalidProxyNumber(
                    "Local proxy numbers must contain '00' after country code".to_string(),
                ));
            }
        }

        Ok(ProxyNumber(cleaned))
    }

    /// Create a local proxy number.
    pub fn new_local(country_code: &str, number_part: &str) -> HeshaResult<Self> {
        let full = format!("+{}00{}", country_code, number_part);
        Self::new(full)
    }

    /// Get the proxy number as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProxyNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SHA256 hash of a salted phone number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneHash(#[serde(with = "hex_serde")] [u8; 32]);

impl PhoneHash {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        PhoneHash(bytes)
    }

    /// Get the hash as bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for PhoneHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// Helper module for hex serialization
mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;

        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid hash length"));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_number_validation() {
        // Valid numbers
        assert!(PhoneNumber::new("+1234567890").is_ok());
        assert!(PhoneNumber::new("+441234567890").is_ok());
        assert!(PhoneNumber::new("+23480475355").is_ok());

        // Invalid numbers
        assert!(PhoneNumber::new("1234567890").is_err()); // Missing +
        assert!(PhoneNumber::new("+123").is_err()); // Too short
        assert!(PhoneNumber::new("+12345678901234567").is_err()); // Too long
        assert!(PhoneNumber::new("+123abc").is_err()); // Non-digits
    }

    #[test]
    fn test_proxy_number_validation() {
        // Valid local proxy
        assert!(ProxyNumber::new("+12001234567890").is_ok());
        assert!(ProxyNumber::new("+442001234567890").is_ok());
        assert!(ProxyNumber::new("+2340012345678").is_ok());

        // Invalid proxy numbers
        assert!(ProxyNumber::new("+99012345678901").is_err()); // 990 not supported
        assert!(ProxyNumber::new("+990123").is_err()); // 990 not supported
        assert!(ProxyNumber::new("+123456789").is_err()); // No 00 marker
    }

    #[test]
    fn test_country_code_extraction() {
        let phone = PhoneNumber::new("+12345678901").unwrap();
        assert_eq!(phone.country_code(), "1");

        let phone = PhoneNumber::new("+442345678901").unwrap();
        assert_eq!(phone.country_code(), "44");
    }
}
