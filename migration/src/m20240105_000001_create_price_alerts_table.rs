use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(PriceAlerts::Table)
                .if_not_exists()
                .col(ColumnDef::new(PriceAlerts::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(PriceAlerts::UserId).string().not_null())
                .col(ColumnDef::new(PriceAlerts::TokenSymbol).string().not_null())
                .col(ColumnDef::new(PriceAlerts::Chain).string().not_null())
                .col(ColumnDef::new(PriceAlerts::TokenAddress).string())
                .col(ColumnDef::new(PriceAlerts::AlertType).string().not_null()) // "above", "below", "percent_change", "portfolio_value"
                .col(ColumnDef::new(PriceAlerts::TargetPrice).decimal())
                .col(ColumnDef::new(PriceAlerts::PercentChange).decimal())
                .col(ColumnDef::new(PriceAlerts::BasePrice).decimal())
                .col(ColumnDef::new(PriceAlerts::Active).boolean().not_null().default(true))
                .col(ColumnDef::new(PriceAlerts::TriggeredAt).timestamp_with_time_zone())
                .col(ColumnDef::new(PriceAlerts::LastCheckedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(PriceAlerts::CreatedAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(PriceAlerts::UpdatedAt).timestamp_with_time_zone().not_null())
                .to_owned()
        ).await?;

        // Create indexes
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_price_alerts_user_id")
                .table(PriceAlerts::Table)
                .col(PriceAlerts::UserId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_price_alerts_active")
                .table(PriceAlerts::Table)
                .col(PriceAlerts::Active)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(PriceAlerts::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum PriceAlerts {
    Table,
    Id,
    UserId,
    TokenSymbol,
    Chain,
    TokenAddress,
    AlertType,
    TargetPrice,
    PercentChange,
    BasePrice,
    Active,
    TriggeredAt,
    LastCheckedAt,
    CreatedAt,
    UpdatedAt,
}
