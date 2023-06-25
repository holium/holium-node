pub mod api;
pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env;

// Define the connection pool type using r2d2
type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

// Create a lazy_static connection pool
lazy_static! {
    static ref POOL: PgPool = {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    };
}

pub fn get_connection() -> Result<PgPooledConnection, String> {
    Ok(POOL.get().map_err(|e| e.to_string())?)
}
