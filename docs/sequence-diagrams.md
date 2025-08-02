# Hesha Protocol Sequence Diagrams

**Author**: Bernard Parah

## Table of Contents

### Issuance Flow
1. [Step 1: User Keypair Generation](#step-1-user-keypair-generation)
2. [Step 2: Phone Verification Request](#step-2-phone-verification-request)
3. [Step 3: OTP Verification](#step-3-otp-verification)
4. [Step 4: Proxy Number Generation](#step-4-proxy-number-generation)
5. [Step 5: Attestation Creation](#step-5-attestation-creation)
6. [Step 6: Saving to Wallet](#step-6-saving-to-wallet)

### Verification Flow
7. [Step 7: Service Integration - Initial Setup](#step-7-service-integration-initial-setup)
8. [Step 8: User Registration with Proxy](#step-8-user-registration-with-proxy)
9. [Step 9: Challenge Creation](#step-9-challenge-creation)
10. [Step 10: Wallet Interaction](#step-10-wallet-interaction)
11. [Step 11: Verification Process](#step-11-verification-process)
12. [Step 12: Offline Verification](#step-12-offline-verification)

### System Overview
13. [Complete System Flow](#complete-system-flow)

---

## Issuance Flow

### Step 1: User Keypair Generation

```mermaid
sequenceDiagram
    participant User
    participant Wallet

    User->>Wallet: Open wallet app
    User->>Wallet: Request new proxy number
    
    Note over Wallet: Generate cryptographic identity
    Wallet->>Wallet: Generate Ed25519 keypair
    Wallet->>Wallet: sk_u = random 32 bytes
    Wallet->>Wallet: pk_u = derive public key
    
    Wallet->>Wallet: Store sk_u securely
    Wallet->>User: Ready to proceed ✓
```

### Step 2: Phone Verification Request

```mermaid
sequenceDiagram
    participant Wallet
    participant Issuer
    participant SMS Gateway

    Wallet->>Wallet: User enters phone: +1234567890
    Wallet->>Wallet: User selects scope: "1" (US)
    
    Wallet->>Issuer: POST /attest
    Note over Wallet,Issuer: {<br/>  phone_number: "+1234567890",<br/>  user_pubkey: "pk_u_base64url",<br/>  scope: "1"<br/>}
    
    Issuer->>Issuer: Validate phone format
    Issuer->>Issuer: Generate OTP: 123456
    Issuer->>SMS Gateway: Send SMS to +1234567890
    SMS Gateway->>Issuer: Delivery confirmed
    
    Issuer->>Wallet: 202 Accepted
    Note over Issuer,Wallet: "OTP sent, awaiting verification"
```

### Step 3: OTP Verification

```mermaid
sequenceDiagram
    participant User
    participant Wallet
    participant Issuer

    User->>User: Receive SMS: "Code: 123456"
    User->>Wallet: Enter OTP: 123456
    
    Wallet->>Issuer: POST /verify-otp
    Note over Wallet,Issuer: {<br/>  otp: "123456",<br/>  session_id: "xyz..."<br/>}
    
    Issuer->>Issuer: Verify OTP matches
    Issuer->>Issuer: Check not expired
    
    alt Valid OTP
        Issuer->>Wallet: 200 OK - Proceed
    else Invalid OTP
        Issuer->>Wallet: 401 - Invalid code
    end
```

### Step 4: Proxy Number Generation

```mermaid
sequenceDiagram
    participant Issuer

    Note over Issuer: Generate proxy deterministically
    
    Issuer->>Issuer: nonce = random 128 bits<br/>"0123456789abcdef0123456789abcdef"
    
    Issuer->>Issuer: Build input string
    Note over Issuer: phone + "|" + user_pubkey + "|" +<br/>domain + "|" + scope + "|" + nonce
    
    Issuer->>Issuer: hash = SHA256(input)
    Issuer->>Issuer: Extract decimal digits
    Note over Issuer: For each hex char: digit = hex % 10
    
    Issuer->>Issuer: proxy = "+100" + first_10_digits
    Note over Issuer: Result: "+10012345678"
```

### Step 5: Attestation Creation

```mermaid
sequenceDiagram
    participant Issuer

    Note over Issuer: Create attestation JWT
    
    Issuer->>Issuer: phone_hash = SHA256("1234567890")
    Note over Issuer: "sha256:c775e7b757ede..."
    
    Issuer->>Issuer: Create binding proof
    Note over Issuer: message = phone_hash + "|" +<br/>user_pubkey + "|" + proxy +<br/>"|" + iat + "|hesha-binding-v2"
    
    Issuer->>Issuer: sig = Sign(sk_i, SHA256(message))
    Issuer->>Issuer: binding_proof = "sig:" + base64url(sig)
    
    Issuer->>Issuer: Build JWT payload
    Note over Issuer: {<br/>  iss: "issuer.com",<br/>  sub: "+10012345678",<br/>  iat: 1720000000,<br/>  exp: 1751536000,<br/>  phone_hash: "sha256:...",<br/>  user_pubkey: "pk_u",<br/>  binding_proof: "sig:...",<br/>  nonce: "0123456789abcdef..."<br/>}
    
    Issuer->>Issuer: Sign JWT with sk_i
```

### Step 6: Saving to Wallet

```mermaid
sequenceDiagram
    participant Issuer
    participant Wallet
    participant Storage

    Issuer->>Wallet: 200 OK
    Note over Issuer,Wallet: {<br/>  proxy_number: "+10012345678",<br/>  attestation: "eyJ0eXAiOiJKV1Q...",<br/>  expires_at: 1751536000<br/>}
    
    Wallet->>Wallet: Validate response
    Wallet->>Wallet: Parse attestation JWT
    
    Wallet->>Storage: Save attestation
    Wallet->>Storage: Map to proxy number
    Wallet->>Storage: Associate with keypair
    
    Wallet->>Wallet: Display success
    Note over Wallet: "Your proxy number:<br/>+10012345678"
```

---

## Verification Flow

### Step 7: Service Integration - Initial Setup

```mermaid
sequenceDiagram
    participant Developer
    participant Service
    participant Code

    Developer->>Code: Install Hesha library
    Note over Code: npm install @hesha/verify
    
    Developer->>Code: Configure verification
    Note over Code: const hesha = new HeshaVerifier({<br/>  serviceName: "app.example.com",<br/>  trustedIssuers: ["issuer.com"],<br/>  challengeExpiry: 300 // 5 minutes<br/>})
    
    Developer->>Service: Deploy integration
    Service->>Service: Ready to verify proxies ✓
```

### Step 8: User Registration with Proxy

```mermaid
sequenceDiagram
    participant User
    participant Service

    User->>Service: Sign up / Log in
    User->>Service: Phone: +10012345678
    
    Service->>Service: Detect proxy pattern
    Note over Service: Regex: /^\+\d{1,4}00\d+$/
    
    Service->>Service: Proxy detected!
    Service->>Service: Initiate verification flow
    Service->>User: "Please verify your phone"
```

### Step 9: Challenge Creation

```mermaid
sequenceDiagram
    participant Service
    participant Database

    Service->>Service: Generate challenge
    Note over Service: {<br/>  service_id: "app.example.com",<br/>  nonce: crypto.random(16),<br/>  timestamp: Date.now(),<br/>  proxy_number: "+10012345678"<br/>}
    
    Service->>Service: Create callback URL
    Note over Service: /verify/callback/abc123
    
    Service->>Database: Store challenge
    Database->>Service: Challenge ID: abc123
    
    Service->>Service: Encode as QR code
    Service->>Service: Display to user
```

### Step 10: Wallet Interaction

```mermaid
sequenceDiagram
    participant User
    participant Wallet
    participant Service

    User->>Wallet: Scan QR code
    Wallet->>Wallet: Decode challenge
    
    Wallet->>Wallet: Find attestation
    Note over Wallet: Lookup by proxy: +10012345678
    
    Wallet->>User: Show approval dialog
    Note over Wallet: "app.example.com wants to<br/>verify your phone number"
    
    User->>Wallet: ✓ Approve
    
    Wallet->>Wallet: Sign challenge
    Note over Wallet: sig = Sign(sk_u, SHA256(challenge))
    
    Wallet->>Service: POST /verify/callback/abc123
    Note over Wallet,Service: {<br/>  attestation: "eyJ0eXA...",<br/>  signature: "sig_base64",<br/>  timestamp: 1720000100<br/>}
```

### Step 11: Verification Process

```mermaid
sequenceDiagram
    participant Service
    participant Cache
    participant Issuer Domain

    Note over Service: Extract issuer from JWT
    Service->>Service: iss = "issuer.com"
    
    Service->>Cache: Get public key for issuer.com
    
    alt Not cached
        Cache->>Issuer Domain: GET /.well-known/hesha/pubkey.json
        Issuer Domain->>Cache: {public_key: "pk_i", ...}
        Cache->>Cache: Store with TTL
    end
    
    Cache->>Service: Return pk_i
    
    Note over Service: Verify attestation
    Service->>Service: Verify JWT signature with pk_i ✓
    Service->>Service: Check expiration ✓
    Service->>Service: Verify binding proof with pk_i ✓
    
    Note over Service: Verify user consent
    Service->>Service: Extract pk_u from attestation
    Service->>Service: Verify challenge signature with pk_u ✓
    Service->>Service: Check nonce not reused ✓
    
    Service->>Service: All checks passed! ✅
```

### Step 12: Offline Verification

```mermaid
sequenceDiagram
    participant User
    participant Service
    participant Local Cache

    Note over Service: Subsequent verification
    Note over Service: (issuer key already cached)
    
    User->>Service: Provide attestation
    
    Service->>Local Cache: Get issuer.com key
    Local Cache->>Service: Return cached pk_i
    
    Note over Service: All verification local
    Service->>Service: Verify JWT ✓
    Service->>Service: Verify binding ✓
    Service->>Service: Verify user sig ✓
    
    Note over Service: Complete in microseconds
    Service->>User: Verified! ✅
    
    Note over Service: No network calls needed
```

---

## System Overview

### Complete System Flow

```mermaid
graph LR
    subgraph "1. Setup"
        A[User] -->|Gets| B[Hesha Wallet]
        B -->|Generates| C[Keypair]
    end
    
    subgraph "2. Issuance"
        C -->|Requests proxy| D[Issuer]
        D -->|Verifies phone| E[SMS/OTP]
        E -->|Confirms| D
        D -->|Issues| F[Attestation]
        F -->|Stored in| B
    end
    
    subgraph "3. Usage"
        B -->|Provides proxy| G[Service A]
        G -->|Challenges| B
        B -->|Signs| H[Response]
        H -->|Verifies| G
    end
    
    subgraph "4. Reuse"
        B -->|Same proxy| I[Service B]
        I -->|Offline verify| I
    end
    
    style A fill:#f9f
    style F fill:#9f9
    style G fill:#9ff
    style I fill:#ff9
```

## Key Points

1. **One-time issuance**: Users verify their phone once, get permanent proxy
2. **Per-service consent**: Each service requires explicit user approval
3. **Offline capability**: After first key fetch, no network needed
4. **Privacy preserved**: Real numbers never shared with services
5. **User control**: Wallet manages all cryptographic operations