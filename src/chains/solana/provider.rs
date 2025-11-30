use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{ Keypair, Signer },
    system_instruction,
    transaction::Transaction,
    hash::Hash,
};
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;

use crate::chains::solana::{ tokens, wallet };
use crate::error::{ AppError, Result };
use crate::providers::{
    Balance,
    ChainProvider,
    TransactionRequest,
    TransactionResponse,
    WalletInfo,
};

pub struct SolanaProvider {
    client: RpcClient,
}

impl SolanaProvider {
    pub fn new(rpc_url: &str) -> Self {
        let client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed()
        );

        Self { client }
    }

    async fn get_spl_token_balance(
        &self,
        wallet_address: &str,
        mint_address: &str
    ) -> Result<Balance> {
        let wallet_pubkey = Pubkey::from_str(wallet_address).map_err(|_| AppError::InvalidAddress)?;
        let mint_pubkey = Pubkey::from_str(mint_address).map_err(|_| AppError::InvalidAddress)?;

        // Get associated token account address
        let token_account = spl_associated_token_account::get_associated_token_address(
            &wallet_pubkey,
            &mint_pubkey
        );

        // Try to fetch token account
        let account_data = self.client.get_account_data(&token_account).await.map_err(|e| {
            // Account doesn't exist means zero balance
            if
                e.to_string().contains("AccountNotFound") ||
                e.to_string().contains("could not find account")
            {
                return AppError::Chain("Token account not found, balance is 0".to_string());
            }
            AppError::Rpc(format!("Failed to get token account: {}", e))
        })?;

        let token_account_info = TokenAccount::unpack(&account_data).map_err(|e|
            AppError::Chain(format!("Failed to parse token account: {}", e))
        )?;

        // Get token info
        let (decimals, symbol) = if let Some(token_info) = tokens::get_token_by_mint(mint_address) {
            (token_info.decimals, token_info.symbol.clone())
        } else {
            (9, "UNKNOWN".to_string()) // Default decimals for unknown tokens
        };

        let balance = (token_account_info.amount as f64) / (10_f64).powi(decimals as i32);

        Ok(Balance {
            balance: balance.to_string(),
            symbol,
            decimals,
        })
    }

    async fn send_spl_token_transaction(
        &self,
        keypair: &Keypair,
        to: Pubkey,
        amount: u64,
        mint: Pubkey,
        compute_units: Option<u32>
    ) -> Result<TransactionResponse> {
        let from_pubkey = keypair.pubkey();

        // Get associated token accounts
        let from_token_account = spl_associated_token_account::get_associated_token_address(
            &from_pubkey,
            &mint
        );
        let to_token_account = spl_associated_token_account::get_associated_token_address(
            &to,
            &mint
        );

        let mut instructions = vec![];

        // Check if destination token account exists, create if not
        if self.client.get_account_data(&to_token_account).await.is_err() {
            let create_account_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &from_pubkey,
                    &to,
                    &mint,
                    &spl_token::id()
                );
            instructions.push(create_account_ix);
        }

        // Add transfer instruction
        let transfer_ix = spl_token::instruction
            ::transfer(
                &spl_token::id(),
                &from_token_account,
                &to_token_account,
                &from_pubkey,
                &[],
                amount
            )
            .map_err(|e| AppError::Chain(format!("Failed to create transfer instruction: {}", e)))?;

        instructions.push(transfer_ix);

        // Get recent blockhash
        let recent_blockhash = self.client
            .get_latest_blockhash().await
            .map_err(|e| AppError::Rpc(format!("Failed to get recent blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&from_pubkey),
            &[keypair],
            recent_blockhash
        );

        let signature = self.client
            .send_and_confirm_transaction(&transaction).await
            .map_err(|e| AppError::Chain(format!("Transaction failed: {}", e)))?;

        Ok(TransactionResponse {
            tx_hash: signature.to_string(),
            status: "confirmed".to_string(),
        })
    }
}

#[async_trait]
impl ChainProvider for SolanaProvider {
    async fn generate_wallet(&self, derivation_index: u32) -> Result<WalletInfo> {
        wallet::generate_wallet(derivation_index)
    }

    async fn restore_wallet(&self, secret: &str, derivation_index: u32) -> Result<WalletInfo> {
        wallet::detect_and_restore(secret, derivation_index)
    }

    async fn get_balance(&self, address: &str) -> Result<Balance> {
        let pubkey = Pubkey::from_str(address).map_err(|_| AppError::InvalidAddress)?;

        let balance: u64 = self.client
            .get_balance(&pubkey).await
            .map_err(|e| AppError::Rpc(format!("Failed to get balance: {}", e)))?;

        let balance_sol = (balance as f64) / (LAMPORTS_PER_SOL as f64);

        Ok(Balance {
            balance: balance_sol.to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
        })
    }

    async fn get_token_balance(&self, address: &str, token_address: &str) -> Result<Balance> {
        self.get_spl_token_balance(address, token_address).await
    }

    async fn send_transaction(
        &self,
        private_key: &str,
        request: TransactionRequest
    ) -> Result<TransactionResponse> {
        let keypair_bytes = bs58
            ::decode(private_key)
            .into_vec()
            .map_err(|_| AppError::InvalidPrivateKey)?;

        let keypair = Keypair::from_bytes(&keypair_bytes).map_err(|_| AppError::InvalidPrivateKey)?;

        let to = Pubkey::from_str(&request.to).map_err(|_| AppError::InvalidAddress)?;

        if let Some(token_address) = request.token_address {
            // SPL token transfer
            let mint = Pubkey::from_str(&token_address).map_err(|_| AppError::InvalidAddress)?;

            // Get token decimals
            let decimals = if let Some(token_info) = tokens::get_token_by_mint(&token_address) {
                token_info.decimals
            } else {
                9 // Default to 9 if unknown
            };

            let amount_float: f64 = request.amount
                .parse()
                .map_err(|_| AppError::InvalidInput("Invalid amount".to_string()))?;
            let amount = (amount_float * (10_f64).powi(decimals as i32)) as u64;

            self.send_spl_token_transaction(&keypair, to, amount, mint, request.compute_units).await
        } else {
            // Native SOL transfer
            let amount_sol: f64 = request.amount
                .parse()
                .map_err(|_| AppError::InvalidInput("Invalid amount".to_string()))?;
            let lamports = (amount_sol * (LAMPORTS_PER_SOL as f64)) as u64;

            let instruction = system_instruction::transfer(&keypair.pubkey(), &to, lamports);

            let recent_blockhash = self.client
                .get_latest_blockhash().await
                .map_err(|e| AppError::Rpc(format!("Failed to get recent blockhash: {}", e)))?;

            let transaction = Transaction::new_signed_with_payer(
                &[instruction],
                Some(&keypair.pubkey()),
                &[&keypair],
                recent_blockhash
            );

            let signature = self.client
                .send_and_confirm_transaction(&transaction).await
                .map_err(|e| AppError::Chain(format!("Transaction failed: {}", e)))?;

            Ok(TransactionResponse {
                tx_hash: signature.to_string(),
                status: "confirmed".to_string(),
            })
        }
    }

    fn validate_address(&self, address: &str) -> bool {
        wallet::validate_address(address)
    }
}
