#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

mod error;
mod models;
mod schema;

use bigdecimal::BigDecimal;
use diesel::{pg::upsert::on_constraint, prelude::*, r2d2::ConnectionManager, PgConnection};
use error::Error;
use near_lake_framework::near_indexer_primitives::{views::SignedTransactionView, CryptoHash};
use r2d2::Pool;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

embed_migrations!("./migrations");

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        let conn = pool.get().unwrap();
        embedded_migrations::run(&conn).unwrap();
        Self { pool }
    }

    pub fn insert_transaction(
        &self,
        transaction: SignedTransactionView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        chunk_index: i32,
        timestamp: BigDecimal,
    ) -> Result<()> {
        use schema::transactions;

        let transaction = models::Transaction {
            hash: transaction.hash.to_string(),
            block_hash: block_hash.to_string(),
            chunk_hash: chunk_hash.to_string(),
            chunk_index,
            timestamp,
            signer_id: transaction.signer_id.to_string(),
            public_key: transaction.public_key.to_string(),
            nonce: transaction.nonce.into(),
            receiver_id: transaction.receiver_id.to_string(),
            signature: transaction.signature.to_string(),
        };
        let conn = self.pool.get().unwrap();
        diesel::insert_into(transactions::table)
            .values(&transaction)
            .on_conflict(on_constraint("transactions_pkey"))
            .do_nothing()
            // .do_update()
            // .set(&transaction)
            .execute(&conn)?;
        Ok(())
    }
}
