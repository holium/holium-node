use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

// A function to establish a connection pool to the SQLite database.
pub fn establish_connection_pool(db_name: &str) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(db_name);
    Ok(Pool::new(manager)?)
}
