# Hesha Protocol Specification

**Version**: 1.1  
**Status**: Definitive  
**Updated**: August 2025 - Signature-based binding proofs  
**Purpose**: Complete protocol definition for cryptographically verifiable proxy phone numbers
**Author**: Bernard Parah

---

## 1. Protocol Overview

### 1.1 What is Hesha?

Hesha is a protocol that enables users to share proxy phone numbers instead of real phone numbers. These proxy numbers are cryptographically verifiable, proving they represent a real phone number without revealing it.

### 1.2 Core Principles

1. **Privacy First**: Real phone numbers are never shared
2. **Cryptographic Proof**: Every proxy number is backed by verifiable attestations
3. **User Control**: Users approve each verification through their wallet
4. **Decentralized Trust**: No central authority required

### 1.3 Key Participants

- **User**: Person who owns a real phone number and wants a proxy
- **Issuer**: Entity that verifies phone number ownership and issues attestations
- **Service**: Application that accepts proxy numbers from users
- **Wallet**: Software that stores attestations and handles verifications

---

## 2. Proxy Number Format

### 2.1 Structure

Proxy numbers follow E.164 format with special prefixes to distinguish them from real numbers.

**Proxy Numbers**
- Format: `+{country_code}00` followed by remaining digits
- Examples:
  - US/Canada: `+10012345678`
  - UK: `+440012345678`
  - Ghana: `+23300549115753`
- The `00` pattern indicates a proxy number

### 2.2 Generation Algorithm

Proxy numbers are generated deterministically using cryptographic hashing:

1. **Inputs**:
   - Phone number (E.164 format)
   - User's Ed25519 public key
   - Issuer domain
   - Scope (1-4 digit calling code)
   - 128-bit random nonce

2. **Process**:
   - Concatenate: `phone_number + "|" + user_pubkey + "|" + issuer_domain + "|" + scope + "|" + nonce`
   - Compute SHA-256 hash
   - Extract decimal digits from hex hash (each hex char modulo 10)
   - Format: `+{scope}00{extracted_digits}`

3. **Number Length**:
   - Formula: `digits_after_00 = max(8, min(10, 15 - len(scope) - 3))`
   - Results in 8-10 digits after "00" depending on country code length

This ensures:
- No collisions (128-bit nonce provides uniqueness)
- Deterministic for same inputs
- Stateless generation (no sequential counters)
- Unpredictable numbers

---

## 3. Attestation Structure

### 3.1 Purpose

An attestation is a cryptographically signed statement that proves:
- A proxy number is valid
- It represents a verified phone number
- It was issued by a trusted issuer

### 3.2 Format

Attestations are JSON Web Tokens (JWTs) with Ed25519 signatures.

**JWT Header**
```json
{
  "alg": "EdDSA",
  "typ": "JWT"
}
```

**JWT Payload**
```json
{
  "iss": "issuer.example.com",
  "sub": "+99012345678901",
  "iat": 1720000000,
  "exp": 1751536000,
  "phone_hash": "sha256:1a2b3c4d5e6f...",
  "user_pubkey": "base64url_encoded_public_key",
  "binding_proof": "sig:SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
  "jti": "550e8400-e29b-41d4",
  "nonce": "0123456789abcdef0123456789abcdef"
}
```

**Required Fields**:
- `iss`: Issuer's domain (for key discovery)
- `sub`: The proxy number
- `iat`: Issued at timestamp (Unix time)
- `exp`: Expiration timestamp (typically 1 year)
- `jti`: JWT ID for uniqueness
- `phone_hash`: SHA-256 hash of normalized phone (format: "sha256:...")
  - Normalization: Remove '+' prefix, ensure only digits
  - Example: "+1234567890" → hash("1234567890")
- `user_pubkey`: User's Ed25519 public key (base64url, no padding)
- `binding_proof`: Ed25519 signature binding (format: "sig:...")
- `nonce`: 128-bit hex nonce used in proxy generation

**Optional Fields**:
- `trust_domain`: For subdomain delegation
- `version`: Protocol version

### 3.3 Issuance Flow

1. **User requests proxy** → Provides real phone number
2. **Issuer verifies ownership** → SMS/call verification
3. **Issuer generates**:
   - Proxy number (deterministic)
   - Attestation JWT (signed by issuer)
4. **User receives** → Proxy number + attestation

### 3.4 Attestation Issuance API

**Endpoint**: `POST /attest`

**Request**:
```json
{
  "phone_number": "+1234567890",        // E.164 format (required)
  "user_pubkey": "base64url_ed25519_key", // Ed25519 public key (base64url, no padding)
  "scope": "<calling-code>"              // Required, 1-4 digit calling code
}
```

**Success Response (200)**:
```json
{
  "proxy_number": "+99012345678901",
  "attestation": "eyJ0eXAiOiJKV1Q...",  // Signed JWT
  "expires_at": 1751536000               // Unix timestamp
}
```

**Error Responses**:
- `400`: Invalid request format
- `401`: Phone verification failed
- `422`: Validation error (invalid phone, key, or scope)
- `429`: Rate limit exceeded
- `500`: Internal server error

**Note**: Phone verification mechanism is implementation-specific and outside the protocol scope.

---

## 4. Verification Protocol

### 4.1 Overview

When a service needs to verify a proxy number, it uses a challenge-response protocol that requires both the issuer's signature (on the attestation) and the user's signature (on the challenge).

### 4.2 Challenge Creation

When a user provides a proxy number to a service:

1. **Service detects proxy** → Recognizes `+990` or `00` pattern
2. **Service creates challenge**:
   ```json
   {
     "proxy_number": "+99012345678901",
     "service_id": "app.example.com",
     "challenge_nonce": "a1b2c3d4e5f6",
     "verification_id": "verify_12345",
     "expires_at": 1720000300,
     "callback_url": "https://app.example.com/callback"
   }
   ```
3. **Service generates QR code** → Contains challenge data

### 4.3 Wallet Response

When user scans QR with their wallet:

1. **Wallet finds attestation** → Matches proxy number
2. **Wallet shows approval** → "App X wants to verify your number"
3. **User approves** → Wallet creates response
4. **Wallet signs challenge**:
   ```json
   {
     "service_id": "app.example.com",
     "challenge_nonce": "a1b2c3d4e5f6",
     "timestamp": 1720000100
   }
   ```
5. **Wallet sends response** → Attestation + signature to callback

### 4.4 Dual Verification

The service performs two verifications:

**Step 1: Verify Issuer's Signature**
1. Extract issuer domain from JWT
2. Fetch public key from `https://{issuer}/.well-known/hesha/pubkey.json`
3. Verify JWT signature with issuer's public key
4. Confirms: Attestation is legitimate

**Step 2: Verify User's Signature**
1. Extract user's public key from attestation
2. Verify challenge response signature
3. Check challenge nonce matches
4. Confirms: User personally approved this verification

Both signatures must be valid for successful verification.

---

## 5. Key Management

### 5.1 Issuer Keys

- **Type**: Ed25519 key pairs
- **Discovery**: Public key at `/.well-known/hesha/pubkey.json`
- **Rotation**: Supported through key versioning

### 5.2 User Keys

- **Type**: Ed25519 key pairs  
- **Generation**: Created by user before attestation request
- **Purpose**: Sign challenge responses
- **Portability**: Can be exported/imported between wallets

### 5.3 Key Discovery

Public keys are discovered via HTTPS:

```
GET https://issuer.example.com/.well-known/hesha/pubkey.json

Response:
{
  "public_key": "base64url_encoded_ed25519_public_key",
  "algorithm": "Ed25519",
  "key_id": "default",
  "created_at": "2024-01-01T00:00:00Z"
}

HTTP Headers:
- Content-Type: application/json
- Cache-Control: public, max-age=3600
```

---

## 6. Security Mechanisms

### 6.1 Challenge-Response

Prevents replay attacks through:
- **Fresh nonces**: Each verification uses unique nonce (minimum 128 bits)
- **Timestamps**: Responses include current timestamp
- **Expiration**: Challenges expire after 5 minutes
- **Nonce tracking**: Services should track used nonces to prevent replay

### 6.2 Binding Proofs

Links phone numbers to proxy numbers cryptographically:
- **Creation**: 
  - Message: `phone_hash + "|" + user_pubkey + "|" + proxy_number + "|" + iat + "|hesha-binding-v2"`
  - Signature: Ed25519-Sign(issuer_private_key, SHA256(message))
  - Output: `"sig:" + base64url(signature)`
- **Verification**: 
  - Reconstruct message from attestation fields
  - Verify signature with issuer's public key
  - Ensures proxy is cryptographically bound to phone hash
- **Privacy**: Phone number never revealed
- **Security**: Publicly verifiable using issuer's public key

### 6.3 Trust Model

- **Issuers are trusted** → To verify phone ownership
- **Services trust issuers** → Through public key infrastructure
- **Users trust wallets** → To store keys securely
- **No central authority** → Fully decentralized

### 6.4 Cryptographic Requirements

- **Signatures**: Ed25519 (256-bit keys)
- **Hashing**: SHA-256
- **HMAC**: HMAC-SHA256 with 256-bit secrets
- **Random Generation**: Cryptographically secure random number generation
- **Encoding**: Base64url (no padding) for all web data
- **Constant-time operations**: Required for all cryptographic comparisons

### 6.5 Input Validation

- **Phone Numbers**: E.164 format, regex: `^\+[1-9]\d{6,14}$`
- **Ed25519 Keys**: 32 bytes, valid curve point, not identity element
- **Proxy Numbers**: Must match `+{scope}00` pattern
- **Scope**: 1-4 digit calling code
- **JWT Size**: Enforce reasonable limits to prevent DoS

---

## 7. Wallet Operations

### 7.1 Core Functions

1. **Store attestations** → Encrypted storage
2. **Scan QR codes** → Decode verification requests
3. **Show approvals** → User consent UI
4. **Sign challenges** → Cryptographic proofs
5. **Send responses** → To service callbacks

### 7.2 Attestation Portability

Users can move attestations between wallets:

**Export Format**
```json
{
  "version": "1.0",
  "attestations": [{
    "proxy_number": "+99012345678901",
    "attestation_jwt": "eyJ0eXAiOiJKV1Q...",
    "user_keypair": {
      "private_key": "encrypted:base64...",
      "public_key": "base64..."
    }
  }]
}
```

---

## 8. Implementation Requirements

### 8.1 Cryptographic Libraries

- Ed25519 signature verification
- SHA-256 hashing
- HMAC computation
- JWT encoding/decoding

### 8.2 Network Requirements

- **HTTPS Required**: All production endpoints MUST use HTTPS
  - Key discovery: `/.well-known/hesha/pubkey.json`
  - Attestation issuance: `/attest`
  - Any other protocol endpoints
- **Development Exception**: HTTP allowed only for localhost
- Support for JSON APIs
- QR code generation/scanning
- Content-Type: `application/json` for all responses

### 8.3 Storage Requirements

- Attestation storage (wallets)
- Private key security (wallets)
- Public key caching (services)

---

## 9. Protocol Flow Summary

1. **Issuance**: User → Issuer → Attestation → Wallet
2. **Usage**: User provides proxy → Service creates challenge
3. **Verification**: Wallet scans → User approves → Dual signatures verified
4. **Completion**: Service accepts proxy as verified contact

This flow ensures:
- Privacy (no real number shared)
- Security (cryptographic proofs)
- Control (user approves each use)
- Simplicity (QR code scanning)

---

## 10. Conclusion

The Hesha Protocol provides a complete system for privacy-preserving phone number verification. By separating identity (real number) from identifier (proxy number) and using cryptographic proofs, users can share contact information without compromising privacy.

This specification defines all necessary components for implementing Hesha-compliant issuers, wallets, and services. The protocol is designed to be simple to understand, secure by default, and respectful of user privacy.

---

**End of Specification**