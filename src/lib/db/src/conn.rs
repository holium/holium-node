use std::fs;

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

// A function to establish a connection pool to the SQLite database.
pub fn establish_connection_pool(db_name: &str) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(db_name);
    Ok(Pool::new(manager)?)
}

pub fn initialize(db_name: &str) -> Result<Pool<SqliteConnectionManager>> {
    let mgr = establish_connection_pool(db_name);
    if mgr.is_ok() {
        // run thru all the sql files in the migrations folder in numerical
        //  order and execute them
        let mut paths: Vec<_> = fs::read_dir("/").unwrap().map(|r| r.unwrap()).collect();
        paths.sort_by_key(|dir| dir.path());
        let conn = mgr.as_ref().unwrap().get().unwrap();
        conn.execute("BEGIN TRANSACTION;", [])?;
        for path in paths {
            // println!("Name: {}", path.path().display())
            // read file contents and execute contents
            let sql = fs::read_to_string(path.path())?;
            conn.execute(sql.as_str(), [])?;
        }
        conn.execute("COMMIT TRANSACTION;", [])?;
    }
    return mgr;
}
