use anyhow::Result;

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

pub struct Db {
    pub pool: Pool<SqliteConnectionManager>,
}

impl Db {
    pub fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        let pool = self.pool.get()?;
        return Ok(pool);
    }
}

// A function to establish a connection pool to the SQLite database.
pub fn initialize(db_name: &str) -> Result<Db> {
    let manager = SqliteConnectionManager::file(db_name);
    Ok(Db {
        pool: Pool::new(manager)?,
    })
}
