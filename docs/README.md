# Crypto Bot

Multi-chain cryptocurrency wallet management system with Telegram bot interface.

## Overview

A production-ready Rust application supporting Ethereum, BSC, and Solana blockchains with encrypted wallet storage, portfolio tracking, and DEX integration.

## Features

### Wallet Management
- Generate wallets with 24-word BIP39 mnemonics
- Restore from mnemonic or private key
- BIP44 derivation paths with custom index support
- AES-256-GCM encrypted storage in PostgreSQL

### Multi-Chain Support
- **ETH** - Ethereum (Mainnet/Sepolia)
- **BSC** - Binance Smart Chain (Mainnet/Testnet)
- **SOLANA** - Solana (Mainnet-Beta/Devnet)

### Transactions
- Send native tokens and ERC20/SPL tokens
- Batch transfers to multiple recipients
- Gas estimation with USD conversion
- Transaction scheduling (one-time and recurring)
- Complete transaction history

### Portfolio & Prices
- Real-time portfolio tracking across all wallets
- USD valuations via CoinGecko API (60s cache)
- 24-hour price change indicators
- Support for top 20 tokens per chain

### DEX Integration
- Uniswap V2 (Ethereum)
- PancakeSwap (BSC)
- Jupiter Aggregator (Solana)
- Slippage protection and price impact limits

### Security
- Per-user rate limiting
- Multi-provider RPC failover
- Environment-based network switching (testnet/mainnet)

## Tech Stack

- **Rust** (stable 1.75+)
- **axum 0.8** - Web framework
- **tokio** - Async runtime
- **sea-orm 1.1** - Database ORM
- **ethers 2.0** - EVM interaction
- **solana-client 3.1** - Solana interaction
- **teloxide** - Telegram bot framework

## Quick Start

```bash
# Clone and configure
git clone <repository-url>
cd crypto-bot
cp .env.example .env

# Edit .env with your settings
# Run
cargo run --release
```

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed setup and [USAGE.md](USAGE.md) for bot commands.

## License

MIT
