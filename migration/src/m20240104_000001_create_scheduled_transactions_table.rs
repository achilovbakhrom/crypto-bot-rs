use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(ScheduledTransaction::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ScheduledTransaction::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT gen_random_uuid()".to_string())
                )
                .col(ColumnDef::new(ScheduledTransaction::UserId).string().not_null())
                .col(ColumnDef::new(ScheduledTransaction::WalletId).uuid().not_null())
                .col(ColumnDef::new(ScheduledTransaction::ToAddress).string().not_null())
                .col(ColumnDef::new(ScheduledTransaction::Amount).string_len(50).not_null())
                .col(ColumnDef::new(ScheduledTransaction::TokenAddress).string().null())
                .col(ColumnDef::new(ScheduledTransaction::ScheduledFor).timestamp().not_null())
                .col(ColumnDef::new(ScheduledTransaction::RecurringType).string_len(20).null())
                .col(ColumnDef::new(ScheduledTransaction::Status).string_len(20).not_null())
                .col(ColumnDef::new(ScheduledTransaction::ExecutedAt).timestamp().null())
                .col(ColumnDef::new(ScheduledTransaction::TxHash).string().null())
                .col(ColumnDef::new(ScheduledTransaction::ErrorMessage).text().null())
                .col(
                    ColumnDef::new(ScheduledTransaction::CreatedAt)
                        .timestamp()
                        .not_null()
                        .extra("DEFAULT NOW()".to_string())
                )
                .col(
                    ColumnDef::new(ScheduledTransaction::UpdatedAt)
                        .timestamp()
                        .not_null()
                        .extra("DEFAULT NOW()".to_string())
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_scheduled_tx_wallet")
                        .from(ScheduledTransaction::Table, ScheduledTransaction::WalletId)
                        .to(Wallet::Table, Wallet::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned()
        ).await?;

        // Create indexes
        manager.create_index(
            Index::create()
                .name("idx_scheduled_tx_user_id")
                .table(ScheduledTransaction::Table)
                .col(ScheduledTransaction::UserId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_scheduled_tx_status")
                .table(ScheduledTransaction::Table)
                .col(ScheduledTransaction::Status)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_scheduled_tx_scheduled_for")
                .table(ScheduledTransaction::Table)
                .col(ScheduledTransaction::ScheduledFor)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(ScheduledTransaction::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum ScheduledTransaction {
    Table,
    Id,
    UserId,
    WalletId,
    ToAddress,
    Amount,
    TokenAddress,
    ScheduledFor,
    RecurringType,
    Status,
    ExecutedAt,
    TxHash,
    ErrorMessage,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Wallet {
    Table,
    Id,
}
