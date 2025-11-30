use axum::{ extract::{ Path, State }, http::StatusCode, Json };
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

use crate::error::Result;
use crate::services::wallet_service::{ GeneratedWalletResponse, RestoredWalletResponse };

use super::AppState;

#[derive(Deserialize)]
pub struct GenerateWalletRequest {
    pub user_id: String,
    pub chain: String,
    #[serde(default)]
    pub derivation_index: Option<u32>,
}

#[derive(Deserialize)]
pub struct RestoreWalletRequest {
    pub user_id: String,
    pub chain: String,
    pub secret: String,
    #[serde(default)]
    pub derivation_index: Option<u32>,
}

pub async fn generate_wallet(
    State(state): State<AppState>,
    Json(request): Json<GenerateWalletRequest>
) -> Result<Json<GeneratedWalletResponse>> {
    let response = state.wallet_service.generate_wallet(
        request.user_id,
        request.chain,
        request.derivation_index
    ).await?;

    Ok(Json(response))
}

pub async fn restore_wallet(
    State(state): State<AppState>,
    Json(request): Json<RestoreWalletRequest>
) -> Result<Json<RestoredWalletResponse>> {
    let response = state.wallet_service.restore_wallet(
        request.user_id,
        request.chain,
        request.secret,
        request.derivation_index
    ).await?;

    Ok(Json(response))
}

pub async fn get_wallet(
    State(state): State<AppState>,
    Path(wallet_id): Path<Uuid>
) -> Result<Json<WalletResponse>> {
    let wallet = state.wallet_service.get_wallet(wallet_id).await?;

    Ok(
        Json(WalletResponse {
            id: wallet.id,
            user_id: wallet.user_id,
            chain: wallet.chain,
            address: wallet.address,
            created_at: wallet.created_at.to_rfc3339(),
        })
    )
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub id: Uuid,
    pub user_id: String,
    pub chain: String,
    pub address: String,
    pub created_at: String,
}
