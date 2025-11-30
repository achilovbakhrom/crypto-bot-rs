use axum::{ extract::{ Path, Query, State }, Json };
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Result;
use crate::providers::Balance;

use super::AppState;

#[derive(Deserialize)]
pub struct BalanceQuery {
    #[serde(default)]
    pub token: Option<String>,
}

pub async fn get_balance(
    State(state): State<AppState>,
    Path(wallet_id): Path<Uuid>,
    Query(query): Query<BalanceQuery>
) -> Result<Json<Balance>> {
    let balance = state.balance_service.get_balance(wallet_id, query.token).await?;

    Ok(Json(balance))
}
