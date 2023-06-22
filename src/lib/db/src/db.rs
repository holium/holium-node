use std::{env, fs};

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Debug, Clone)]
pub struct Db<'a> {
    db_name: &'a str,
    pool: Option<Pool<SqliteConnectionManager>>,
}

impl Db<'_> {
    pub async fn new(db_name: &str) -> Result<Db> {
        Ok(Db {
            db_name,
            pool: None,
        })
    }

    // A function to establish a connection pool to the SQLite database.
    fn establish_connection(&mut self) -> Result<()> {
        Pool::new(SqliteConnectionManager::file(self.db_name))?;
        return Ok(());
    }

    /// - Creates a new SQLite database with the given name
    /// - Executes all the SQL files in the migrations folder (sorted by name)
    async fn initialize(&mut self) -> Result<()> {
        if self.establish_connection().is_ok() {
            // run thru all the sql files in the migrations folder in numerical
            //  order and execute them
            println!("{:?}", env::current_dir());
            let mut paths: Vec<_> = fs::read_dir("src/lib/db/src/migrations")
                .unwrap()
                .map(|r| r.unwrap())
                .collect();
            paths.sort_by_key(|dir| dir.path());
            // let conn = mgr.as_ref().unwrap().get().unwrap();
            self.execute_batch("BEGIN TRANSACTION;")?;
            for path in paths {
                println!("processing sql file '{}'...", path.path().display());
                // read file contents and execute contents
                let sql = fs::read_to_string(path.path())?;
                self.execute_batch(sql.as_str())?;
            }
            self.execute_batch("COMMIT TRANSACTION;")?;
        }
        return Ok(());
    }

    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        let conn = self.pool.as_ref().unwrap().get().unwrap();
        return Ok(conn.execute_batch(sql)?);
    }
}

pub async fn initialize(db_name: &str) -> Result<Db> {
    let mut db = Db::new(db_name).await?;
    db.initialize().await?;
    return Ok(db);
}
