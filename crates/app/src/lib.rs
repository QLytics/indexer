use actix_cors::Cors;
use actix_web::{dev::Server, middleware, web, App, HttpServer};
use near_ql_db::Database;
use std::env;

#[cfg(debug_assertions)]
const LOG_LEVEL: &str = "actix_server=debug,actix_web=debug";
#[cfg(not(debug_assertions))]
const LOG_LEVEL: &str = "actix_server=info,actix_web=info";

pub async fn main(database: web::Data<Database>) -> std::io::Result<Server> {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", LOG_LEVEL);
    env_logger::init();

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .supports_credentials()
                    .max_age(3600),
            )
    })
    .bind(format!("0.0.0.0:{}", env::var("PORT").unwrap()))?
    .run())
}
