use anyhow::Result;

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<SqliteConnectionManager>,
}

impl Db {
    pub fn new(db_name: &str) -> Db {
        let mut db_path = db_name.to_string();
        if db_path != ":memory:" {
            db_path = format!("src/lib/db/data/{}.sqlite", db_name);
        }
        let manager = SqliteConnectionManager::file(db_path.as_str());
        let pool = Pool::new(manager);
        if pool.is_err() {
            panic!("libdb: [new] Pool::new call failed");
        }
        Db {
            pool: pool.unwrap(),
        }
    }

    pub fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        let pool = self.pool.get()?;
        return Ok(pool);
    }
}

// A function to establish a connection pool to the SQLite database.
pub fn initialize(db_name: &str) -> Result<Db> {
    let mut db_path = db_name.to_string();
    if db_path != ":memory:" {
        db_path = format!("src/lib/db/data/{}.sqlite", db_name);
    }
    let manager = SqliteConnectionManager::file(db_path.as_str());
    Ok(Db {
        pool: Pool::new(manager)?,
    })
}
