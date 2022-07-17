#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate strum;

mod error;
mod models;
mod schema;
pub(crate) mod util;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = Arc<RwLock<Database>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub use models::{
    execution_outcomes::{ExecutionOutcome, ExecutionOutcomeReceipt},
    receipts::{DataReceipt, Receipt},
    transaction_actions::TransactionAction,
    transactions::Transaction,
    ExecutionOutcomeStatus,
};

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

    pub fn insert_receipts(&mut self, receipts: &Vec<Receipt>) -> Result<()> {
        use schema::receipts;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_or_ignore_into(receipts::table)
            .values(receipts)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_data_receipts(&mut self, data_receipts: &Vec<DataReceipt>) -> Result<()> {
        use schema::data_receipts;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_or_ignore_into(data_receipts::table)
            .values(data_receipts)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_transactions(&mut self, transactions: &Vec<Transaction>) -> Result<()> {
        use schema::transactions;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_or_ignore_into(transactions::table)
            .values(transactions)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_transaction_actions(
        &mut self,
        transaction_actions: &Vec<TransactionAction>,
    ) -> Result<()> {
        use schema::transaction_actions;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_or_ignore_into(transaction_actions::table)
            .values(transaction_actions)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_execution_outcomes(
        &mut self,
        execution_outcomes: &Vec<ExecutionOutcome>,
    ) -> Result<()> {
        use schema::execution_outcomes;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_or_ignore_into(execution_outcomes::table)
            .values(execution_outcomes)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }

    pub fn insert_execution_outcome_receipts(
        &mut self,
        execution_outcome_receipts: &Vec<ExecutionOutcomeReceipt>,
    ) -> Result<()> {
        use schema::execution_outcome_receipts;

        let mut conn = self.pool.get().unwrap();
        diesel::insert_or_ignore_into(execution_outcome_receipts::table)
            .values(execution_outcome_receipts)
            .execute(&mut conn)
            .unwrap();
        Ok(())
    }
}
