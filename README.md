# Crypto Bot - Multi-Chain Wallet Management System

A production-ready Rust-based cryptocurrency wallet management system supporting multiple blockchains (Ethereum, BSC, Solana) with encrypted storage, multi-provider RPC failover, and REST API.

## Features

- **Multi-Chain Support**: Ethereum, BSC (EVM), and Solana
- **Wallet Operations**: Generate (24-word mnemonic) and restore wallets (mnemonic/private key)
- **Encrypted Storage**: AES-256-GCM encryption for private keys in PostgreSQL
- **Token Support**: Native tokens + top 20 ERC20/SPL tokens + custom token addresses
- **RPC Failover**: Automatic failover across multiple providers
- **BIP44 Derivation**: Standard derivation paths with custom index support
- **Rate Limiting**: Per-user rate limiting
- **Network Switching**: Environment-based testnet/mainnet configuration

## Tech Stack

- **Rust** (stable) with async/await
- **axum 0.8** - Web framework
- **tokio** - Async runtime
- **sea-orm 1.1** - Database ORM
- **ethers 2.0** - EVM blockchain interaction
- **solana-client 3.1** - Solana blockchain interaction (nonblocking)
- **bip39** - Mnemonic generation
- **aes-gcm** - Encryption

## Prerequisites

- Rust 1.75+ (stable)
- PostgreSQL 14+
- Docker (optional, for development)

## Quick Start

1. **Clone and setup**:
   ```bash
   git clone <repository-url>
   cd crypto-bot
   cp .env.example .env
   ```

2. **Configure environment**:
   Edit `.env` and set:
   - `DATABASE_URL`
   - `ENCRYPTION_KEY` (generate with: `openssl rand -hex 32`)
   - RPC URLs for your chains
   - `NETWORK_MODE=testnet` (or `mainnet`)

3. **Setup database**:
   ```bash
   # Start PostgreSQL (if using Docker)
   docker run -d --name postgres \
     -e POSTGRES_PASSWORD=password \
     -e POSTGRES_DB=crypto_bot \
     -p 5432:5432 postgres:14

   # Run migrations (automatically on startup, or manually)
   cargo run --bin migration
   ```

4. **Run the service**:
   ```bash
   cargo run --release
   ```

   Server starts at `http://localhost:8080`

## API Endpoints

### Generate New Wallet
```bash
POST /api/wallets/generate
Content-Type: application/json

{
  "user_id": "user123",
  "chain": "ETH",
  "derivation_index": 0  // optional, default: 0
}

Response:
{
  "id": "uuid",
  "address": "0x...",
  "mnemonic": "word1 word2 ... word24",
  "chain": "ETH"
}
```

### Restore Wallet
```bash
POST /api/wallets/restore
Content-Type: application/json

{
  "user_id": "user123",
  "chain": "SOLANA",
  "secret": "word1 word2 ... word24",  // or private key
  "derivation_index": 0  // optional, default: 0
}

Response:
{
  "id": "uuid",
  "address": "...",
  "chain": "SOLANA"
}
```

### Get Balance
```bash
GET /api/wallets/{wallet_id}/balance?token=0x...

Response:
{
  "balance": "1.234567890123456789",
  "symbol": "ETH",
  "decimals": 18
}
```

### Send Transaction
```bash
POST /api/wallets/{wallet_id}/transfer
Content-Type: application/json

{
  "to": "0x...",
  "amount": "0.1",
  "token": "0x...",  // optional, for token transfers
  "max_fee_per_gas": "50000000000",  // optional, EVM only
  "max_priority_fee_per_gas": "2000000000",  // optional, EVM only
  "compute_units": 200000  // optional, Solana only
}

Response:
{
  "tx_hash": "0x...",
  "status": "pending"
}
```

## Supported Chains

- **ETH** - Ethereum
- **BSC** - Binance Smart Chain
- **SOLANA** - Solana

## Error Responses

```json
{
  "error": {
    "code": "INVALID_ADDRESS",
    "message": "Invalid address format",
    "field": "to"
  }
}
```

## Development

```bash
# Run tests
cargo test

# Run with logs
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT
