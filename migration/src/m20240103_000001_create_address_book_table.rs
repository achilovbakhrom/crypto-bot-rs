use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(AddressBook::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AddressBook::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                        .extra("DEFAULT gen_random_uuid()".to_string())
                )
                .col(ColumnDef::new(AddressBook::UserId).string().not_null())
                .col(ColumnDef::new(AddressBook::Name).string_len(100).not_null())
                .col(ColumnDef::new(AddressBook::Address).string().not_null())
                .col(ColumnDef::new(AddressBook::Chain).string_len(20).not_null())
                .col(ColumnDef::new(AddressBook::Notes).text().null())
                .col(
                    ColumnDef::new(AddressBook::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .extra("DEFAULT NOW()".to_string())
                )
                .col(
                    ColumnDef::new(AddressBook::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .extra("DEFAULT NOW()".to_string())
                )
                .to_owned()
        ).await?;

        // Create indexes
        manager.create_index(
            Index::create()
                .name("idx_address_book_user_id")
                .table(AddressBook::Table)
                .col(AddressBook::UserId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name("idx_address_book_user_name")
                .table(AddressBook::Table)
                .col(AddressBook::UserId)
                .col(AddressBook::Name)
                .unique()
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(AddressBook::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum AddressBook {
    Table,
    Id,
    UserId,
    Name,
    Address,
    Chain,
    Notes,
    CreatedAt,
    UpdatedAt,
}
