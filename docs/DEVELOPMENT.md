# Local Development Guide

## Prerequisites

- Rust 1.75+ (stable)
- Go 1.21+
- Docker

## Setup

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Database Setup

```bash
docker compose up -d
```

### 3. Configure Environment

```bash
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=postgresql://postgres:password@localhost:6432/crypto_bot
ENCRYPTION_KEY=<generate with: openssl rand -hex 32>
NETWORK_MODE=testnet

# RPC URLs
ETH_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
BSC_RPC_URL=https://data-seed-prebsc-1-s1.binance.org:8545
SOLANA_RPC_URL=https://api.devnet.solana.com

# Telegram (optional)
TELEGRAM_BOT_TOKEN=your_bot_token
```

### 4. Run Migrations

```bash
make migrate
```

Other migration commands:

```bash
make migrate-status  # Check migration status
make migrate-down    # Rollback last migration
make migrate-fresh   # Drop all and re-run migrations
```

### 5. Run the Application

```bash
make dev    # Development with debug logs
make run    # Run without debug logs
make build  # Release build
```

## Development Commands

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Check compilation
cargo check
```

## Database Management

```bash
make db-up      # Start database
make db-down    # Stop database
make db-reset   # Reset database (delete all data)
make db-logs    # View database logs

# Check tables
docker compose exec postgres psql -U postgres crypto_bot -c "\dt"
```

## Project Structure

```
crypto-bot/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library exports
│   ├── config.rs         # Configuration
│   ├── error.rs          # Error types
│   ├── api/              # REST API handlers
│   ├── bot/              # Telegram bot
│   ├── chains/           # Blockchain providers
│   │   ├── evm/          # Ethereum/BSC
│   │   └── solana/       # Solana
│   ├── db/               # Database entities
│   ├── dex/              # DEX integrations
│   ├── providers/        # Chain abstraction
│   └── services/         # Business logic
├── migration/            # Database migrations
├── tools/                # DevOps tools (Go)
├── tests/                # Integration tests
└── docs/                 # Documentation
```

## Troubleshooting

**Database connection failed**

```bash
# Check if PostgreSQL is running
docker compose ps

# View logs
docker compose logs postgres
```

**RPC errors**

- Verify RPC URLs in `.env`
- Try alternative public endpoints
- Check rate limits

**Bot not responding**

- Verify `TELEGRAM_BOT_TOKEN`
- Check logs: `RUST_LOG=debug cargo run`
