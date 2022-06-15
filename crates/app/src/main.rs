use actix_web::web;
use near_ql_db::Database;
use std::env;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database = web::Data::new(Database::new(&env::var("DATABASE_URL").unwrap()));
    near_ql_indexer::start_indexing(database.clone());
    near_ql::main(database).await?.await
}
