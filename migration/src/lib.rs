pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_wallets_table;
mod m20240102_000001_create_transactions_table;
mod m20240103_000001_create_address_book_table;
mod m20240104_000001_create_scheduled_transactions_table;
mod m20240105_000001_create_price_alerts_table;
mod m20240105_000002_create_security_settings_table;
mod m20240106_000001_create_swaps_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_wallets_table::Migration),
            Box::new(m20240102_000001_create_transactions_table::Migration),
            Box::new(m20240103_000001_create_address_book_table::Migration),
            Box::new(m20240104_000001_create_scheduled_transactions_table::Migration),
            Box::new(m20240105_000001_create_price_alerts_table::Migration),
            Box::new(m20240105_000002_create_security_settings_table::Migration),
            Box::new(m20240106_000001_create_swaps_table::Migration)
        ]
    }
}
