use std::sync::Arc;

pub mod wallet;
pub mod balance;
pub mod transfer;
pub mod transaction;

use crate::services::{ BalanceService, TransferService, WalletService, TransactionService };

#[derive(Clone)]
pub struct AppState {
    pub wallet_service: Arc<WalletService>,
    pub balance_service: Arc<BalanceService>,
    pub transfer_service: Arc<TransferService>,
    pub transaction_service: Arc<TransactionService>,
}

impl AppState {
    pub fn new(
        wallet_service: Arc<WalletService>,
        balance_service: Arc<BalanceService>,
        transfer_service: Arc<TransferService>,
        transaction_service: Arc<TransactionService>
    ) -> Self {
        Self {
            wallet_service,
            balance_service,
            transfer_service,
            transaction_service,
        }
    }
}
