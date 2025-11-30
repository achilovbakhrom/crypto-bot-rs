use axum::{ extract::{ Path, State }, Json };
use uuid::Uuid;

use crate::error::Result;
use crate::providers::TransactionResponse;
use crate::services::transfer_service::TransferRequest;

use super::AppState;

pub async fn send_transaction(
    State(state): State<AppState>,
    Path(wallet_id): Path<Uuid>,
    Json(request): Json<TransferRequest>
) -> Result<Json<TransactionResponse>> {
    let response = state.transfer_service.send_transaction(wallet_id, request).await?;

    Ok(Json(response))
}
