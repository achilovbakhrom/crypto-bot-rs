use axum::{ extract::{ Path, Query, State }, Json };
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

use crate::error::Result;
use crate::db::entity::transaction;

use super::AppState;

#[derive(Deserialize)]
pub struct TransactionQueryParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Deserialize)]
pub struct UserTransactionQueryParams {
    pub user_id: String,
    pub chain: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub async fn get_wallet_transactions(
    State(state): State<AppState>,
    Path(wallet_id): Path<Uuid>,
    Query(params): Query<TransactionQueryParams>
) -> Result<Json<Vec<TransactionResponse>>> {
    let transactions = state.transaction_service.get_wallet_transactions(
        wallet_id,
        params.limit,
        params.offset
    ).await?;

    let response: Vec<TransactionResponse> = transactions
        .into_iter()
        .map(|tx| tx.into())
        .collect();

    Ok(Json(response))
}

pub async fn get_user_transactions(
    State(state): State<AppState>,
    Query(params): Query<UserTransactionQueryParams>
) -> Result<Json<Vec<TransactionResponse>>> {
    let transactions = state.transaction_service.get_user_transactions(
        &params.user_id,
        params.chain.as_deref(),
        params.limit,
        params.offset
    ).await?;

    let response: Vec<TransactionResponse> = transactions
        .into_iter()
        .map(|tx| tx.into())
        .collect();

    Ok(Json(response))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(tx_hash): Path<String>
) -> Result<Json<TransactionResponse>> {
    let transaction = state.transaction_service.get_transaction_by_hash(&tx_hash).await?;

    Ok(Json(transaction.into()))
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub tx_hash: String,
    pub chain: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub token_address: Option<String>,
    pub token_symbol: Option<String>,
    pub status: String,
    pub block_number: Option<i64>,
    pub gas_used: Option<String>,
    pub created_at: String,
}

impl From<transaction::Model> for TransactionResponse {
    fn from(tx: transaction::Model) -> Self {
        Self {
            id: tx.id,
            wallet_id: tx.wallet_id,
            tx_hash: tx.tx_hash,
            chain: tx.chain,
            from_address: tx.from_address,
            to_address: tx.to_address,
            amount: tx.amount,
            token_address: tx.token_address,
            token_symbol: tx.token_symbol,
            status: tx.status,
            block_number: tx.block_number,
            gas_used: tx.gas_used,
            created_at: tx.created_at.to_string(),
        }
    }
}
