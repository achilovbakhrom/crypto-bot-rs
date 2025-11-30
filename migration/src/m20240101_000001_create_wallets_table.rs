use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Wallet::Table)
                .if_not_exists()
                .col(ColumnDef::new(Wallet::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Wallet::UserId).string().not_null())
                .col(ColumnDef::new(Wallet::Chain).string().not_null())
                .col(ColumnDef::new(Wallet::Address).string().not_null())
                .col(ColumnDef::new(Wallet::EncryptedPrivateKey).text().not_null())
                .col(
                    ColumnDef::new(Wallet::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp())
                )
                .to_owned()
        ).await?;

        // Create index on user_id and chain
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_wallets_user_chain")
                .table(Wallet::Table)
                .col(Wallet::UserId)
                .col(Wallet::Chain)
                .to_owned()
        ).await?;

        // Create index on address
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_wallets_address")
                .table(Wallet::Table)
                .col(Wallet::Address)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Wallet::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Wallet {
    Table,
    Id,
    UserId,
    Chain,
    Address,
    EncryptedPrivateKey,
    CreatedAt,
}
