#[macro_use]
extern crate diesel;

#[macro_use]
extern crate strum;

mod error;
mod models;
mod schema;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = Arc<RwLock<Database>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub use models::Transaction;

use diesel::{prelude::*, r2d2::ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use error::Error;
use r2d2::Pool;
use std::sync::{Arc, RwLock};

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
}
