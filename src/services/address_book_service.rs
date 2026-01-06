use std::sync::Arc;
use uuid::Uuid;
use sea_orm::*;

use crate::db::entity::address_book;
use crate::db::entity::address_book::Entity as AddressBook;
use crate::error::{ AppError, Result };

pub struct AddressBookService {
    db: Arc<DatabaseConnection>,
}

impl AddressBookService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Save a new address to the address book
    pub async fn save_address(
        &self,
        user_id: String,
        name: String,
        address: String,
        chain: String,
        notes: Option<String>
    ) -> Result<address_book::Model> {
        // Check if name already exists for this user
        let existing = AddressBook::find()
            .filter(address_book::Column::UserId.eq(&user_id))
            .filter(address_book::Column::Name.eq(&name))
            .one(self.db.as_ref()).await?;

        if existing.is_some() {
            return Err(AppError::InvalidInput(format!("Address name '{}' already exists", name)));
        }

        let now = chrono::Utc::now();
        let address_book = address_book::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            name: Set(name),
            address: Set(address),
            chain: Set(chain),
            notes: Set(notes),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        let result = AddressBook::insert(address_book).exec_with_returning(self.db.as_ref()).await?;

        Ok(result)
    }

    /// Update an existing address
    pub async fn update_address(
        &self,
        user_id: &str,
        name: &str,
        new_address: Option<String>,
        new_chain: Option<String>,
        new_notes: Option<String>
    ) -> Result<address_book::Model> {
        let address_book = AddressBook::find()
            .filter(address_book::Column::UserId.eq(user_id))
            .filter(address_book::Column::Name.eq(name))
            .one(self.db.as_ref()).await?
            .ok_or_else(|| AppError::NotFound("Address not found".to_string()))?;

        let mut active_model: address_book::ActiveModel = address_book.into();

        if let Some(addr) = new_address {
            active_model.address = Set(addr);
        }
        if let Some(chain) = new_chain {
            active_model.chain = Set(chain);
        }
        if let Some(notes) = new_notes {
            active_model.notes = Set(Some(notes));
        }
        active_model.updated_at = Set(chrono::Utc::now().into());

        let result = active_model.update(self.db.as_ref()).await?;
        Ok(result)
    }

    /// Delete an address from the address book
    pub async fn delete_address(&self, user_id: &str, name: &str) -> Result<()> {
        let result = AddressBook::delete_many()
            .filter(address_book::Column::UserId.eq(user_id))
            .filter(address_book::Column::Name.eq(name))
            .exec(self.db.as_ref()).await?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound("Address not found".to_string()));
        }

        Ok(())
    }

    /// Get an address by name
    pub async fn get_address(&self, user_id: &str, name: &str) -> Result<address_book::Model> {
        AddressBook::find()
            .filter(address_book::Column::UserId.eq(user_id))
            .filter(address_book::Column::Name.eq(name))
            .one(self.db.as_ref()).await?
            .ok_or_else(|| AppError::NotFound(format!("Address '{}' not found", name)))
    }

    /// List all addresses for a user
    pub async fn list_addresses(
        &self,
        user_id: &str,
        chain: Option<&str>
    ) -> Result<Vec<address_book::Model>> {
        let mut query = AddressBook::find().filter(address_book::Column::UserId.eq(user_id));

        if let Some(chain) = chain {
            query = query.filter(address_book::Column::Chain.eq(chain));
        }

        let addresses = query.order_by_asc(address_book::Column::Name).all(self.db.as_ref()).await?;

        Ok(addresses)
    }

    /// Search addresses by partial name match
    pub async fn search_addresses(
        &self,
        user_id: &str,
        search_term: &str
    ) -> Result<Vec<address_book::Model>> {
        let addresses = AddressBook::find()
            .filter(address_book::Column::UserId.eq(user_id))
            .filter(address_book::Column::Name.contains(search_term))
            .order_by_asc(address_book::Column::Name)
            .all(self.db.as_ref()).await?;

        Ok(addresses)
    }
}
