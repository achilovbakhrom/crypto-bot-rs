use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(SecuritySettings::Table)
                .if_not_exists()
                .col(ColumnDef::new(SecuritySettings::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(SecuritySettings::UserId).string().not_null().unique_key())
                .col(ColumnDef::new(SecuritySettings::PinHash).string())
                .col(
                    ColumnDef::new(SecuritySettings::PinEnabled).boolean().not_null().default(false)
                )
                .col(
                    ColumnDef::new(SecuritySettings::ConfirmationDelaySeconds)
                        .integer()
                        .not_null()
                        .default(0)
                )
                .col(ColumnDef::new(SecuritySettings::DailyWithdrawalLimit).decimal())
                .col(ColumnDef::new(SecuritySettings::WeeklyWithdrawalLimit).decimal())
                .col(ColumnDef::new(SecuritySettings::RequireConfirmationAbove).decimal())
                .col(
                    ColumnDef::new(SecuritySettings::SessionTimeout)
                        .integer()
                        .not_null()
                        .default(3600)
                ) // seconds
                .col(ColumnDef::new(SecuritySettings::LastActivity).timestamp_with_time_zone())
                .col(
                    ColumnDef::new(SecuritySettings::WalletLocked)
                        .boolean()
                        .not_null()
                        .default(false)
                )
                .col(
                    ColumnDef::new(SecuritySettings::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SecuritySettings::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                )
                .to_owned()
        ).await?;

        // Create withdrawal tracking table
        manager.create_table(
            Table::create()
                .table(WithdrawalTracking::Table)
                .if_not_exists()
                .col(ColumnDef::new(WithdrawalTracking::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(WithdrawalTracking::UserId).string().not_null())
                .col(ColumnDef::new(WithdrawalTracking::Amount).decimal().not_null())
                .col(ColumnDef::new(WithdrawalTracking::TokenSymbol).string().not_null())
                .col(ColumnDef::new(WithdrawalTracking::UsdValue).decimal().not_null())
                .col(
                    ColumnDef::new(WithdrawalTracking::Timestamp)
                        .timestamp_with_time_zone()
                        .not_null()
                )
                .to_owned()
        ).await?;

        // Create index on user_id and timestamp for withdrawal tracking
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_withdrawal_tracking_user_timestamp")
                .table(WithdrawalTracking::Table)
                .col(WithdrawalTracking::UserId)
                .col(WithdrawalTracking::Timestamp)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(WithdrawalTracking::Table).to_owned()).await?;

        manager.drop_table(Table::drop().table(SecuritySettings::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum SecuritySettings {
    Table,
    Id,
    UserId,
    PinHash,
    PinEnabled,
    ConfirmationDelaySeconds,
    DailyWithdrawalLimit,
    WeeklyWithdrawalLimit,
    RequireConfirmationAbove,
    SessionTimeout,
    LastActivity,
    WalletLocked,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum WithdrawalTracking {
    Table,
    Id,
    UserId,
    Amount,
    TokenSymbol,
    UsdValue,
    Timestamp,
}
