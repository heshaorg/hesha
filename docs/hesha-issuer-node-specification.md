# Hesha Issuer Node Specification

**Version**: 1.0  
**Status**: Definitive  
**Purpose**: Protocol requirements for entities that issue Hesha attestations
**Author**: Bernard Parah

---

## 1. Overview

### 1.1 Role in the Protocol

The Issuer Node:
- Verifies ownership of phone numbers (method unspecified)
- Generates proxy numbers deterministically
- Issues cryptographically signed attestations
- Serves public keys for verification

### 1.2 Protocol Requirements

An issuer MUST:
1. Verify phone ownership before issuing attestations
2. Generate proxy numbers using the specified algorithm
3. Sign attestations with Ed25519 keys
4. Make public keys discoverable via HTTPS
5. Include all required fields in attestations

---

## 2. Proxy Number Generation

### 2.1 Algorithm Requirements

Proxy numbers MUST be generated using this exact algorithm:

**Step 1: Validate Inputs**
```
// Validate protocol version
if (version != "1.0") {
    throw UnsupportedVersionError("Only version 1.0 is supported")
}

// Validate phone number (E.164)
if (!phone_number.matches("^\+[1-9]\d{6,14}$")) {
    throw InvalidPhoneNumberError
}

// Validate user public key (Ed25519)
// 1. Decode from base64url (no padding)
// 2. Check length is exactly 32 bytes
// 3. Verify it's a valid Ed25519 point
if (!validate_ed25519_pubkey(user_pubkey)) {
    throw InvalidPublicKeyError
}

// Validate scope (1-4 digit calling code)
if (!scope.matches("^[1-9]\d{0,3}$")) {
    throw InvalidScopeError
}
```

**Step 2: Construct Input String**
```
input = phone_number + "|" + user_pubkey + "|" + issuer_domain + "|" + scope + "|" + nonce
```

Required format:
- phone_number: E.164 format with '+' (e.g., "+1234567890")
- user_pubkey: Base64-encoded Ed25519 public key
- issuer_domain: The issuer's domain (e.g., "example.com")
- scope: Country calling code (1-4 digits, e.g., "1", "44", "233", "1264", "990")
- nonce: 128-bit random value, lowercase hex encoded (32 characters)
- separator: Pipe character "|" (ASCII 0x7C)

**Step 3: Generate Hash**
```
hash_bytes = SHA256(UTF-8(input))
```

**Step 4: Extract Digits**
```
// Extract more digits than needed to ensure we have enough
digits = ""
hex_string = hex_encode(hash_bytes)  // 64 hex characters

for i from 0 to 63:  // Process all hex characters
    hex_digit = hex_string[i]
    decimal_digit = int(hex_digit, 16) % 10
    digits += str(decimal_digit)
    
    if len(digits) >= 20:  // More than enough for any country code
        break
```

This ensures we always have sufficient digits by using the hex representation, which gives us up to 64 opportunities to extract digits.

**Step 5: Format as Proxy Number**
```
cc = scope  // Calling code (1-4 digits like "1", "44", "1264", "990")

// Calculate digits after "00" pattern
digits_after_00 = max(8, min(10, 15 - len(cc) - 3))

// Practical result:
// 1-2 digit country codes: 10 digits after 00
// 3 digit country codes: 9 digits after 00  
// 4 digit country codes: 8 digits after 00

proxy_number = "+" + cc + "00" + digits[0:digits_after_00]
```

This ensures:
- Minimum 100 million unique combinations (8 digits)
- Maximum 10 billion combinations where possible
- Realistic phone number lengths
- Full E.164 compliance (≤15 total characters)

**Step 6: Final Validation**
```
// Ensure result is E.164 compliant
if (len(proxy_number) > 15) {
    throw ProxyNumberTooLongError  // Should never happen with correct algorithm
}

return proxy_number
```

Note: The 128-bit random nonce provides sufficient entropy to make collisions astronomically unlikely (1 in 2^128), eliminating the need for collision checking.

### 2.2 Number Formats

**Proxy Number Format**:
- Format: `+{country_code}00` followed by remaining digits
- The `00` pattern after any country code identifies it as a proxy number
- Examples:
  - `+10012345678` (US format)
  - `+440012345678` (UK format)
  - `+99001234567890` (using unassigned +990)
  - `+12640012345` (4-digit country code)

### 2.3 Uniqueness Guarantee

The algorithm provides both cryptographic uniqueness and sufficient entropy:

**Collision Resistance**:
- 128-bit random nonce: ~1 in 2^128 collision probability
- Deterministic generation: same inputs always produce same output
- Each user can have multiple proxy numbers (different nonces)

**Number Space by Country Code Length**:
- 1-2 digit codes: 10 digits after "00" = 10 billion combinations
- 3 digit codes: 9 digits after "00" = 1 billion combinations
- 4 digit codes: 8 digits after "00" = 100 million combinations

Note: While the protocol operates statelessly, implementers may choose to track issued numbers for business purposes (customer support, analytics, compliance).

---

## 3. Attestation Structure

### 3.1 Required JWT Claims

```json
{
  "iss": "issuer.example.com",     // Issuer domain
  "sub": "+99012345678901",        // Proxy number
  "iat": 1720000000,               // Issued at (Unix timestamp)
  "exp": 1751536000,               // Expiration timestamp
  "jti": "unique-jwt-id",          // JWT ID for uniqueness
  "phone_hash": "sha256:...",      // Hash of real phone number (see 3.2)
  "user_pubkey": "base64url...",   // User's Ed25519 public key (base64url, no padding)
  "binding_proof": "sig:...",      // Ed25519 signature binding (v1.1)
  "nonce": "0123456789abcdef..."   // 128-bit hex nonce used in proxy generation (32 chars)
}
```

### 3.2 Phone Hash Computation

The phone_hash MUST be computed as follows:

```
1. Start with E.164 phone number (e.g., "+1234567890")
2. Remove the '+' prefix: "1234567890"
3. Ensure only digits remain (no spaces, dashes, or formatting)
4. Compute: hash_bytes = SHA256(UTF-8(normalized_number))
5. Format: phone_hash = "sha256:" + hex_encode(hash_bytes)
```

**Example**:
```
Input: "+1 (234) 567-8900" or "+1234567890"
Normalized: "1234567890"
UTF-8 bytes: [0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30]
SHA256: 0xc775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646
Result: "sha256:c775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646"
```

### 3.3 Nonce Field

The `nonce` field in the JWT contains the 128-bit random value that was used during proxy number generation. This field is REQUIRED and serves two purposes:

1. **Reproducibility**: Allows anyone to verify the proxy number was generated correctly by recomputing the hash with the same inputs
2. **Uniqueness**: Ensures that even if the same phone number is attested multiple times by the same issuer, different proxy numbers are generated

The nonce MUST be the exact same value used in the proxy number generation algorithm (Section 2.2).

### 3.4 Optional JWT Claims

```json
{
  "trust_domain": "parent.com",    // For subdomain delegation
  "version": "1.0"                 // Protocol version
}
```

### 3.5 Binding Proof (v1.1 - Signature Based)

The binding_proof MUST be computed as:
```
binding_proof = "sig:" + base64url(Ed25519-Sign(issuer_private_key, SHA256(message)))

where message = phone_hash + "|" + user_pubkey + "|" + proxy_number + "|" + iat + "|hesha-binding-v2"
```

**Input Formats**:
- `issuer_private_key`: Ed25519 private key (same key used for JWT signing)
- `phone_hash`: The complete phone hash string including prefix (e.g., "sha256:c775e7b757...")
- `user_pubkey`: Base64url-encoded Ed25519 public key as it appears in the JWT (no padding)
- `proxy_number`: The full proxy number with '+' prefix (e.g., "+99012345678901")
- `iat`: Unix timestamp as decimal string (e.g., "1720000000")
- `"hesha-binding-v2"`: Version string for protocol v1.1
- `|`: Pipe separator for canonical message format

**Example**:
```
Inputs:
- phone_hash: "sha256:c775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646"
- user_pubkey: "MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E="
- proxy_number: "+99012345678901"
- iat: "1720000000"

Message: "sha256:c775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646|MCowBQYDK2VwAyEAa7bsa2eI7T6w9P6KVJdLvmSGq2uPmTqz2R0RBAl6R2E=|+99012345678901|1720000000|hesha-binding-v2"

SHA256(message): [32 bytes]
Ed25519-Sign(key, hash): [64 bytes]
Result: "sig:SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
```

This ensures each attestation has a cryptographically verifiable binding between all components.

**Binding proof**: The binding proof is  publicly verifiable using the issuer's Ed25519 public key. Services can verify:
1. The JWT signature with the issuer's public key
2. The binding signature with the same public key
3. All JWT claims and expiration

The signature-based binding proof provides:
- Publicly verifiable attestation integrity
- Cryptographic proof that components belong together
- Protection against proxy number substitution
- Complete verification without secrets

### 3.5 JWT Signature

- Algorithm: EdDSA (Ed25519)
- Format: Standard JWT (header.payload.signature)
- Encoding: Base64url without padding

### 3.6 Encoding Standards

**Base64url Usage**:
All base64-encoded values in this protocol MUST use base64url encoding (RFC 4648, Section 5):
- URL and filename safe alphabet
- No padding characters ('=')
- Used for: public keys, JWT segments, any web-transmitted data

**Why base64url**:
- Safe in URLs without escaping
- Safe in JSON without escaping  
- Standard for JWTs and web protocols
- Avoids '+' and '/' characters that require special handling

---

## 4. Key Management

### 4.1 Key Requirements

- Algorithm: Ed25519
- Key size: 256 bits
- Generation: Cryptographically secure random

### 4.2 Public Key Discovery

Issuers MUST serve their public key at:
```
https://{issuer_domain}/.well-known/hesha/pubkey.json
```

Required format:
```json
{
  "public_key": "base64url_encoded_ed25519_public_key",  // base64url, no padding
  "algorithm": "Ed25519",
  "key_id": "key_identifier",
  "created_at": "ISO8601_timestamp"
}
```

### 4.3 Key Security

**Ed25519 Signing Key**:
- Same key used for both JWT signatures and binding proofs
- MUST be protected from unauthorized access
- MUST never be exposed in logs or errors
- Key rotation affects both JWT and binding signatures

---

## 5. API Requirements

### 5.1 Attestation Issuance

Issuers MUST provide an attestation endpoint that:
1. Accepts user's public key and phone number
2. Returns a signed attestation after verification

**Required Endpoint**: POST /attest

**Request Format**:
```json
{
  "phone_number": "+1234567890",        // E.164 format (required)
  "user_pubkey": "base64url_ed25519_key", // Ed25519 public key (base64url, no padding)
  "scope": "<calling-code>"              // Required, 1-4 digit calling code
}
```

**Phone Number Validation**:
The phone_number MUST pass E.164 validation:
- Starts with '+' followed by country code
- Contains only digits after '+'
- Total length between 9-15 characters
- Valid regex: `^\+[1-9]\d{8,14}$`

**Scope Requirements**:
- Must be provided (required parameter)
- Must be 1-4 digit calling code (e.g., "1", "44", "1264", "990")
- Can be any valid or unassigned calling code
- Does not need to match the phone's country code

**Success Response (200)**:
```json
{
  "proxy_number": "+99012345678901",
  "attestation": "eyJ0eXAiOiJKV1Q...",  // Signed JWT
  "expires_at": 1751536000               // Unix timestamp
}
```

**Error Responses**:

HTTP Status Codes:
- 400: Malformed request
- 401: Phone verification required/failed
- 422: Invalid phone number or public key format
- 429: Rate limit exceeded
- 500: Internal server error

**Error Response Format**:
```json
{
  "error": "<error_code>",              // Machine-readable error code
  "error_description": "<description>"  // Human-readable description
}
```

Example error codes:
- `invalid_version`: Unsupported protocol version
- `invalid_phone_number`: Phone number failed E.164 validation
- `invalid_public_key`: Public key failed Ed25519 validation
- `invalid_scope`: Scope not a valid 1-4 digit code
- `verification_failed`: Phone ownership verification failed

Note: How phone verification is performed is outside the protocol scope.

### 5.2 Public Key Endpoint

Required endpoint:
```
GET /.well-known/hesha/pubkey.json
```

Must return:
- Valid JSON matching the key discovery format
- Content-Type: application/json
- Cache headers appropriate for key rotation policy

---

## 6. Security Requirements

### 6.1 Transport Security

- Production: HTTPS required for all endpoints
- Development: HTTP permitted for localhost only
- Public key endpoint MUST always be HTTPS in production

### 6.2 Phone Number Privacy

Issuers MUST:
- Validate phone numbers are E.164 compliant before processing
- Never include real phone numbers in attestations
- Only store phone number hashes if needed
- Use SHA-256 for phone number hashing

### 6.3 User Key Handling

Issuers MUST validate user-provided Ed25519 public keys:

**Required Validations**:
```
1. Decode from base64url (no padding)
2. Verify length is exactly 32 bytes
3. Verify the key represents a valid Ed25519 point:
   - Not the identity element (all zeros)
   - Is a valid point on the Ed25519 curve
   - Has correct order

// Most Ed25519 libraries handle point validation automatically
// Example validation:
try {
    let decoded = base64url_decode(user_pubkey)  // no padding
    if (decoded.length != 32) throw InvalidKeyLength
    
    // This will fail if not a valid point
    let key = Ed25519PublicKey::from_bytes(decoded)
    
    // Additional check for identity element
    if (decoded.all_zeros()) throw InvalidKeyIdentity
    
    return true
} catch {
    return false
}
```

**Security Note**: 
- Invalid public keys can lead to signature forgery or key substitution attacks
- Most reputable Ed25519 libraries (libsodium, ed25519-dalek) perform point validation automatically
- NEVER accept malformed or invalid public keys

Users are responsible for their own key generation and management.

---

## 7. Phone Verification

The protocol does not specify how phone ownership must be verified. Issuers may use any method that provides appropriate assurance, such as:
- SMS verification codes
- Voice calls
- App-based verification
- Carrier APIs

The verification method is an implementation choice outside the protocol scope.

---

## 8. Example Implementation Flow

```
1. User generates Ed25519 keypair (keeps private key)
2. User provides phone number + public key to issuer
3. Issuer verifies phone ownership (method varies)
4. Issuer generates proxy number:
   - nonce = random_128_bits() // lowercase hex, 32 chars
   - scope = user_requested_scope  // Required parameter
   - proxy = generate_proxy(phone, pubkey, domain, scope, nonce)
5. Issuer creates attestation:
   - Set all required claims with user's public key
   - Compute binding proof including user's key
   - Sign with issuer key
6. Return attestation to user
```

---

**This specification defines only the protocol requirements for issuer nodes. Implementation details are provided as guidance only. The protocol ensures interoperability while allowing implementation flexibility.**

*End of Specification*