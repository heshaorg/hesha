# Product Requirements Document: Hesha Demo Issuer Website

## Executive Summary
A demonstration issuer website that showcases the Hesha Protocol's proxy phone number issuance flow, including payment processing via Bitcoin/Lightning Network or USDT. This demo illustrates the protocol's value proposition where users pay for privacy while services verify for free.

## Goals
- Demonstrate end-to-end Hesha Protocol flow
- Show payment model: users pay, services verify free
- Provide simple, accessible interface for everyday users
- Create compelling demo for potential partners/investors

## User Stories

### End User Flow
1. User visits demo issuer website
2. Enters their phone number and public key
3. Receives OTP prompt (hardcoded: 1234)
4. Sees payment options: $1 in BTC/LN/USDT
5. Completes payment
6. Receives their proxy number and attestation
7. Can download/copy attestation for wallet import

### Key Features

#### Landing Page
- Clear explanation of Hesha Protocol
- "Get Your Private Number" CTA
- Simple 3-step process visualization
- Price clearly shown: $1

#### Phone Verification Flow
```
Page 1: Enter Your Details
- Input 1: Phone number with country selector
  * Validation: E.164 format
  * Auto-detect country code
- Input 2: Your public key
  * Placeholder: "Your Hesha public key (base64url)"
  * Validation: Valid Ed25519 public key format
  * Link: "Don't have a key? Open Hesha Wallet"
    - onClick: window.open('http://localhost:5173', '_blank', 'noopener,noreferrer,private')
    - Opens wallet in new private window for testing
- Note: "We'll verify this is your number"

Page 2: Enter OTP
- Input: 6-digit code field
- Hardcoded acceptance: "1234"
- Resend option (mock)
- Shows: "Verifying +234801234567"

Page 3: Payment
- Amount: $1 USD equivalent
- Options:
  * Bitcoin (on-chain) - QR code + address
  * Lightning Network - Lightning invoice
  * USDT (TRC-20) - Address + QR
- Payment detection via webhook/polling
```

#### Success Page
- Display proxy number prominently
- Show attestation details:
  * Issuer: demo.hesha.org
  * Expires: 30 days
  * Your proxy: +234001234567
- Actions:
  * Copy attestation JWT
  * Download as .hesha file
  * "Add to Wallet" button (opens wallet if installed)
  * QR code for mobile scanning

## Technical Architecture

### Frontend (Simple HTML/CSS/JS)
- Plain HTML pages
- Simple CSS for styling (can use a minimal framework like Pico.css)
- Vanilla JavaScript for interactions
- Pages:
  * index.html - Landing page
  * verify.html - Phone/key entry + OTP
  * payment.html - Payment selection
  * success.html - Show attestation

### Backend (Node.js + Express + SQLite)
```javascript
// Simple SQLite schema
sessions: id, phone_hash, user_pubkey, status, created_at
payments: id, session_id, status, created_at

// API Endpoints
POST /api/verify-phone
- Hash phone number with SHA256
- Store hash + pubkey in sessions table
- Keep phone in server memory only for OTP flow
- Returns session ID

POST /api/verify-otp
- Accepts "1234" only
- Updates session status to 'verified'

POST /api/create-payment
- Creates payment record
- Returns mock payment addresses

GET /api/payment-status/:id
- Auto-completes after 5-10 seconds
- Updates payment status in DB

POST /api/issue-attestation
- Fetches session data (hash + pubkey)
- Retrieves phone from memory cache
- Calls issuer node's /attest endpoint
- Clears phone from memory after use
- Returns proxy number and JWT
```

### Payment Integration (Mocked)
- **Bitcoin**: Display demo address: `bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh`
- **Lightning**: Show demo invoice: `lnbc1...` (shortened for display)
- **USDT**: Display demo TRC-20 address: `TN9RRaVkdEW3FKbWzgXJHKXM3UyxK5zPHa`
- Auto-confirm payment after 5-10 seconds with loading animation
- Show "Payment received!" confirmation

### Integration with Issuer Node
- Hesha issuer node runs on **http://localhost:3000**
- Demo backend connects to issuer node's /attest endpoint
- Backend runs on different port (8080) to avoid conflicts
- Flow: Frontend (8080) → Backend (8080) → Issuer Node (3000)

## UI/UX Design

### Design Principles
- Clean, trustworthy appearance
- Mobile-first responsive
- Minimal steps to completion
- Clear progress indication

### Key Screens
1. **Landing**: Hero, explanation, CTA
2. **Phone Entry**: Simple form with country selector
3. **OTP**: 6 boxes, auto-focus, auto-submit
4. **Payment**: Clear options, status updates
5. **Success**: Celebration, clear next steps

### Color Scheme
- Primary: Hesha brand color
- Success: Green for completion
- Accent: Blue for CTAs
- Neutral: Grays for text/borders

## Website Copy
See `issuer-demo-copy.md` for all user-facing text and messaging.

## Project Structure
```
hesha-issuer-demo/
├── public/
│   ├── index.html         # Landing page
│   ├── verify.html        # Phone + key entry
│   ├── payment.html       # Payment selection
│   ├── success.html       # Show results
│   ├── style.css          # Simple styles
│   └── app.js             # Frontend logic
├── server.js              # Express server (runs on port 8080)
├── db.js                  # SQLite setup
├── package.json
└── .env                   # ISSUER_NODE_URL=http://localhost:3000
```

## Development Phases

### Phase 1: Core Flow (Week 1)
- Phone verification UI
- OTP hardcoded flow
- Basic attestation display
- Integration with issuer node

### Phase 2: Payment (Week 2)
- Payment UI components
- Mock payment processing
- Payment status polling
- Success animations

### Phase 3: Polish (Week 3)
- Error handling
- Loading states
- Mobile optimization
- Copy/download functionality

## Success Metrics
- Complete flow in <2 minutes
- Clear understanding of value prop
- Successful attestation issuance
- Payment flow completion

## Future Enhancements (Post-Demo)
- Real SMS verification
- Production payment processing
- Multiple language support
- User attestation history
- Email attestation delivery

## Open Questions
1. Bitnob integration for African Bitcoin payments?
2. Custom domain for demo (demo.hesha.org)?
3. Analytics/tracking for demo metrics?
4. Support for test payments on testnet?

## Acceptance Criteria
- [ ] User can complete full flow
- [ ] Attestation imports to wallet
- [ ] Payment options clearly presented
- [ ] Mobile-responsive design
- [ ] Error states handled gracefully
- [ ] Clear value proposition communicated