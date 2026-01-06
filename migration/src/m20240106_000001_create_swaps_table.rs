use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Swaps::Table)
                .if_not_exists()
                .col(ColumnDef::new(Swaps::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Swaps::UserId).string().not_null())
                .col(ColumnDef::new(Swaps::WalletId).uuid().not_null())
                .col(ColumnDef::new(Swaps::Chain).string().not_null())
                .col(ColumnDef::new(Swaps::Dex).string().not_null()) // "uniswap_v2", "uniswap_v3", "pancakeswap", "jupiter"
                .col(ColumnDef::new(Swaps::FromToken).string().not_null())
                .col(ColumnDef::new(Swaps::FromTokenAddress).string().null())
                .col(ColumnDef::new(Swaps::ToToken).string().not_null())
                .col(ColumnDef::new(Swaps::ToTokenAddress).string().null())
                .col(ColumnDef::new(Swaps::FromAmount).decimal().not_null())
                .col(ColumnDef::new(Swaps::ToAmount).decimal().not_null())
                .col(ColumnDef::new(Swaps::ExpectedToAmount).decimal().null()) // Expected amount before execution
                .col(ColumnDef::new(Swaps::PriceImpact).decimal().null()) // Price impact percentage
                .col(ColumnDef::new(Swaps::Slippage).decimal().not_null()) // User-set slippage tolerance
                .col(ColumnDef::new(Swaps::TxHash).string().null())
                .col(ColumnDef::new(Swaps::Status).string().not_null()) // "pending", "success", "failed"
                .col(ColumnDef::new(Swaps::ErrorMessage).string().null())
                .col(ColumnDef::new(Swaps::GasFee).decimal().null())
                .col(ColumnDef::new(Swaps::Route).json().null()) // Swap route information (for multi-hop swaps)
                .col(
                    ColumnDef::new(Swaps::CreatedAt)
                        .timestamp()
                        .default(Expr::current_timestamp())
                        .not_null()
                )
                .col(
                    ColumnDef::new(Swaps::UpdatedAt)
                        .timestamp()
                        .default(Expr::current_timestamp())
                        .not_null()
                )
                .to_owned()
        ).await?;

        // Create indexes
        manager.create_index(
            Index::create()
                .name("idx_swaps_user_id")
                .table(Swaps::Table)
                .col(Swaps::UserId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_swaps_wallet_id")
                .table(Swaps::Table)
                .col(Swaps::WalletId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_swaps_tx_hash")
                .table(Swaps::Table)
                .col(Swaps::TxHash)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Swaps::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Swaps {
    Table,
    Id,
    UserId,
    WalletId,
    Chain,
    Dex,
    FromToken,
    FromTokenAddress,
    ToToken,
    ToTokenAddress,
    FromAmount,
    ToAmount,
    ExpectedToAmount,
    PriceImpact,
    Slippage,
    TxHash,
    Status,
    ErrorMessage,
    GasFee,
    Route,
    CreatedAt,
    UpdatedAt,
}
