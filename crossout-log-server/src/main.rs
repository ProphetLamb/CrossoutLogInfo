use std::env;

use actix_web::{
    middleware,
    App,  HttpServer,
};
use diesel::r2d2::{ConnectionManager, Pool};

use crossout_log_server::endpoints::*;
use crossout_log_server::db::*;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate diesel_migrations;

diesel_migrations::embed_migrations!("./migrations");

fn get_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| format!(
        "postgres://{}:{}@{}/{}",
        env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string()),
        env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
        env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("POSTGRES_DB").unwrap_or_else(|_| "crossout-log-server".to_string())
    ))
}

fn apply_migrations(pool: &DbPool) {
    println!("Applying migrations");
    let conn = pool.get().expect("Fail to get pool");
    diesel_migrations::run_pending_migrations(&conn).expect("Failed to apply migrations");
}

fn get_pool(db_url: String) -> DbPool {
    let manager = ConnectionManager::<DbConnection>::new(db_url);
    Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to init pool")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Initializing crossout-log-server");

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let pool = get_pool(get_url());
    apply_migrations(&pool);

    let data = get_app_state(pool);
    let my_url = env::var("MY_URL").unwrap_or_else(|_| "127.0.0.1:8088".into());

    println!("Launching HTTP server at {}", my_url);
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(middleware::Logger::default())
            .configure(configure_endpoints)
    })
        .bind(&my_url)
        .expect("Failed to start server")
        .run().await?;

    println!("Stopped HTTP server at {}", my_url);
    Ok(())
}
