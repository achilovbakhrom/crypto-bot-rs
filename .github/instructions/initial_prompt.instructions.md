---
applyTo: "**"
---

Build a Rust cryptocurrency wallet management system. Phase 1: Core wallet operations.

REQUIREMENTS:

- Multi-chain support: EVM chains (Ethereum, BSC) and Solana
- Generate wallets with BIP39 mnemonics
- Import existing wallets (mnemonic/private key)
- Store encrypted private keys in PostgreSQL
- Check balances (native + ERC20/SPL tokens)
- Send transactions (native + tokens)
- Multi-provider RPC failover (Alchemy, Infura, Ankr free tiers)
- REST API with proper error handling

TECH STACK:

- Rust stable (async)
- axum web framework
- tokio async runtime
- sqlx for PostgreSQL (async)
- ethers-rs for EVM chains
- solana-client + solana-sdk for Solana
- bip39 for mnemonic generation
- aes-gcm or ring for encryption
- serde for JSON
- dotenv for config

ARCHITECTURE (modular monolith):
src/
├── main.rs
├── config.rs // Environment config
├── api/ // REST endpoints
│ ├── mod.rs
│ ├── wallet.rs
│ ├── balance.rs
│ └── transfer.rs
├── domain/ // Core business logic
│ ├── mod.rs
│ ├── wallet.rs
│ └── transaction.rs
├── services/ // Application services
│ ├── mod.rs
│ ├── wallet_service.rs
│ ├── balance_service.rs
│ └── transfer_service.rs
├── providers/ // Chain abstraction
│ ├── mod.rs
│ ├── chain_provider.rs // Trait
│ ├── evm_provider.rs
│ └── solana_provider.rs
├── rpc/ // RPC manager
│ ├── mod.rs
│ └── rpc_manager.rs // Failover logic
├── db/ // Database
│ ├── mod.rs
│ ├── models.rs
│ └── repository.rs
└── crypto/ // Encryption utilities
├── mod.rs
└── encryption.rs

DATABASE SCHEMA (PostgreSQL):
CREATE TABLE wallets (
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
user_id VARCHAR(255) NOT NULL,
chain VARCHAR(20) NOT NULL, -- 'ETH', 'BSC', 'SOLANA'
address VARCHAR(255) NOT NULL,
encrypted_private_key TEXT NOT NULL,
created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_user_chain ON wallets(user_id, chain);
CREATE INDEX idx_address ON wallets(address);

CHAIN PROVIDER TRAIT: #[async_trait]
pub trait ChainProvider: Send + Sync {
async fn generate_wallet(&self) -> Result<Wallet>;
async fn import_wallet(&self, key: &str) -> Result<Wallet>;
async fn get_balance(&self, address: &str) -> Result<Balance>;
async fn get_token_balance(&self, address: &str, token: &str) -> Result<Balance>;
async fn send_transaction(&self, tx: Transaction) -> Result<TxHash>;
}

RPC ENDPOINTS:
POST /api/wallets/generate // {chain: "ETH|BSC|SOLANA"}
POST /api/wallets/import // {chain, mnemonic|private_key}
GET /api/wallets/:id/balance // ?token=0x... (optional)
POST /api/wallets/:id/transfer // {to, amount, token?}

SECURITY:

- Use AES-256-GCM for key encryption
- Store encryption key in environment (.env)
- Never log private keys or mnemonics
- Validate all addresses before operations
- Use prepared statements (sqlx prevents SQL injection)

RPC PROVIDERS CONFIG (.env):
ETH_RPC_URLS=https://eth.llamarpc.com,https://rpc.ankr.com/eth
BSC_RPC_URLS=https://bsc-dataseed.binance.org,https://rpc.ankr.com/bsc
SOLANA_RPC_URLS=https://api.mainnet-beta.solana.com,https://rpc.ankr.com/solana

Start with:

1. Project setup with Cargo.toml dependencies
2. Database models and migrations
3. Chain provider trait and basic EVM implementation
4. Encryption utilities

Use production-grade error handling with thiserror. Write clean, idiomatic Rust with proper async/await patterns.
