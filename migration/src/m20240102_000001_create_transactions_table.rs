use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Transaction::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Transaction::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT gen_random_uuid()".to_string())
                )
                .col(ColumnDef::new(Transaction::WalletId).uuid().not_null())
                .col(ColumnDef::new(Transaction::TxHash).string().not_null())
                .col(ColumnDef::new(Transaction::Chain).string_len(20).not_null())
                .col(ColumnDef::new(Transaction::FromAddress).string().not_null())
                .col(ColumnDef::new(Transaction::ToAddress).string().not_null())
                .col(ColumnDef::new(Transaction::Amount).string_len(50).not_null())
                .col(ColumnDef::new(Transaction::TokenAddress).string().null())
                .col(ColumnDef::new(Transaction::TokenSymbol).string_len(20).null())
                .col(ColumnDef::new(Transaction::Status).string_len(20).not_null())
                .col(ColumnDef::new(Transaction::BlockNumber).big_integer().null())
                .col(ColumnDef::new(Transaction::GasUsed).string_len(50).null())
                .col(
                    ColumnDef::new(Transaction::CreatedAt)
                        .timestamp()
                        .not_null()
                        .extra("DEFAULT NOW()".to_string())
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_transaction_wallet")
                        .from(Transaction::Table, Transaction::WalletId)
                        .to(Wallet::Table, Wallet::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned()
        ).await?;

        // Create indexes for faster lookups
        manager.create_index(
            Index::create()
                .name("idx_transaction_wallet_id")
                .table(Transaction::Table)
                .col(Transaction::WalletId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_transaction_tx_hash")
                .table(Transaction::Table)
                .col(Transaction::TxHash)
                .unique()
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_transaction_created_at")
                .table(Transaction::Table)
                .col(Transaction::CreatedAt)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Transaction::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Transaction {
    Table,
    Id,
    WalletId,
    TxHash,
    Chain,
    FromAddress,
    ToAddress,
    Amount,
    TokenAddress,
    TokenSymbol,
    Status,
    BlockNumber,
    GasUsed,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Wallet {
    Table,
    Id,
}
