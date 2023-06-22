use std::{env, fs};

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

// A function to establish a connection pool to the SQLite database.
pub fn establish_connection_pool(db_name: &str) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(db_name);
    Ok(Pool::new(manager)?)
}

/// - Creates a new SQLite database with the given name
/// - Executes all the SQL files in the migrations folder (sorted by name)
pub fn initialize(db_name: &str) -> Result<Pool<SqliteConnectionManager>> {
    let mgr = establish_connection_pool(db_name);
    if mgr.is_ok() {
        // run thru all the sql files in the migrations folder in numerical
        //  order and execute them
        println!("{:?}", env::current_dir());
        let mut paths: Vec<_> = fs::read_dir("src/lib/db/src/migrations")
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        paths.sort_by_key(|dir| dir.path());
        let conn = mgr.as_ref().unwrap().get().unwrap();
        conn.execute_batch("BEGIN TRANSACTION;")?;
        for path in paths {
            println!("processing sql file '{}'...", path.path().display());
            // read file contents and execute contents
            let sql = fs::read_to_string(path.path())?;
            conn.execute_batch(sql.as_str())?;
        }
        conn.execute_batch("COMMIT TRANSACTION;")?;
    }
    return mgr;
}
