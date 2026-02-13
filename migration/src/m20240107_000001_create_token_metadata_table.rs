use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TokenMetadata::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TokenMetadata::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(TokenMetadata::Chain).string().not_null())
                    .col(ColumnDef::new(TokenMetadata::ContractAddress).string().not_null())
                    .col(ColumnDef::new(TokenMetadata::Symbol).string().not_null())
                    .col(ColumnDef::new(TokenMetadata::Name).string().not_null())
                    .col(ColumnDef::new(TokenMetadata::Decimals).small_integer().not_null())
                    .col(ColumnDef::new(TokenMetadata::LogoUrl).string().null())
                    .col(ColumnDef::new(TokenMetadata::CoingeckoId).string().null())
                    .col(
                        ColumnDef::new(TokenMetadata::IsVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(TokenMetadata::DiscoveredAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TokenMetadata::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint on (chain, contract_address)
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_token_metadata_chain_address")
                    .table(TokenMetadata::Table)
                    .col(TokenMetadata::Chain)
                    .col(TokenMetadata::ContractAddress)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on (chain, symbol) for lookup
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_token_metadata_chain_symbol")
                    .table(TokenMetadata::Table)
                    .col(TokenMetadata::Chain)
                    .col(TokenMetadata::Symbol)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TokenMetadata::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TokenMetadata {
    Table,
    Id,
    Chain,
    ContractAddress,
    Symbol,
    Name,
    Decimals,
    LogoUrl,
    CoingeckoId,
    IsVerified,
    DiscoveredAt,
    UpdatedAt,
}
