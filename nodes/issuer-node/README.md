# Hesha Issuer Node

Reference implementation of a Hesha Protocol issuer node.

## Quick Start

1. First, run the setup command to create your issuer configuration:
   ```bash
   hesha setup
   ```

2. Start the issuer node with your configuration:
   ```bash
   # Using config directory
   HESHA_CONFIG_DIR=~/.hesha/issuer/default cargo run --bin hesha-issuer-node
   
   # Or using config file path directly
   CONFIG_PATH=~/.hesha/issuer/default/config/issuer.toml cargo run --bin hesha-issuer-node
   ```

## Configuration

The issuer node loads configuration from `issuer.toml` created by the setup command. The configuration includes:

- **Identity**: Issuer name, trust domain, and public key
- **Port**: Server port (default: 3000)
- **Attestation validity**: How long attestations remain valid (default: 365 days)

The private key is loaded from `keys/private.key` relative to the config file.

## Endpoints

- `POST /attest` - Request attestation with user public key
- `POST /attest/simple` - Request attestation with verification code
- `GET /.well-known/hesha/pubkey.json` - Public key discovery

## Environment Variables

- `HESHA_CONFIG_DIR` - Directory containing config/issuer.toml
- `CONFIG_PATH` - Direct path to issuer.toml file
- `BIND_ADDRESS` - Override bind address (fallback only)
- `ISSUER_DOMAIN` - Override domain (fallback only)
- `PRIVATE_KEY_PATH` - Override private key path (fallback only)