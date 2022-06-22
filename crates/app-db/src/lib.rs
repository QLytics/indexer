#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate strum;

mod error;
mod models;
mod schema;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = Arc<RwLock<Database>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub use models::{Receipt, Transaction, TransactionAction};

use diesel::{prelude::*, r2d2::ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use error::Error;
use parking_lot::RwLock;
use r2d2::Pool;
use std::sync::Arc;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(url: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        let mut conn = pool.get().unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        Self { pool }
    }

    pub fn insert_receipt(&mut self, receipt: models::Receipt) -> Result<()> {
        use schema::receipts;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(receipts::table)
            .values(&receipt)
            .on_conflict(receipts::id)
            .do_nothing()
            // .do_update()
            // .set(&transaction)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_transaction(&mut self, transaction: models::Transaction) -> Result<()> {
        use schema::transactions;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(transactions::table)
            .values(&transaction)
            .on_conflict(transactions::hash)
            .do_nothing()
            // .do_update()
            // .set(&transaction)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_transaction_action(
        &mut self,
        transaction_action: models::TransactionAction,
    ) -> Result<()> {
        use schema::transaction_actions;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(transaction_actions::table)
            .values(&transaction_action)
            .on_conflict(transaction_actions::hash)
            .do_nothing()
            // .do_update()
            // .set(&transaction)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }
}
