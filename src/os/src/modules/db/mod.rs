use std::{collections::HashMap, env, fs};

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

use anyhow::{bail, Result};

// use self::contracts::IdentitySystem;

pub mod providers;
// use providers::chat::ChatDataProvider;

// pub type DataProviders = HashMap::<String, Box<dyn providers::types::DataProvider + Sync + Send>>;

// pub type ImportData = fn(db: DbModule) -> Result<()>;
pub type DataProviders = HashMap<String, fn(db: &Db) -> Result<()>>;

pub struct Db {
    pool: Pool<SqliteConnectionManager>,
}

impl Db {
    // pub fn initialize(&mut self) -> Result<&DbModule> {
    //     let manager = SqliteConnectionManager::file(":memory:");
    //     self.pool = Some(Pool::new(manager)?);

    //     // add some data providers (e.g. chat-db data from a ship)
    //     let mut data_providers =
    //         HashMap::<String, Box<dyn providers::types::DataProvider + Sync + Send>>::new();
    //     data_providers.insert(String::from("chat-db"), Box::new(ChatDataProvider::new()));
    //     self.data_providers = Some(data_providers);

    //     // self.initialized = true;
    //     return Ok(self);
    // }

    pub fn get_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        return self.pool.get().unwrap();
    }

    // genesis - should be called one-time at first-time holon boot sequence
    //  1) create initial database schema by running the scripts
    //     found in the ./sql folder in sequence (by name)
    //  2) for each data provider that is registered:
    //     call the data provider's import_data method passing this
    //     module as an input
    // pub fn invoke_genesis(&self) -> Result<()> {
    //     let mut result = self.generate_schema();
    //     if result.is_err() {
    //         return result;
    //     }
    //     return self.import_data();
    // }

    // fn import_data(&self) -> Result<()> {
    //     if self.data_providers.is_none() {
    //         bail!("db: [import_data] module is in an invalid state");
    //     }
    //     for pair in self.data_providers.as_ref().unwrap() {
    //         println!("importing data from {}...", pair.0);
    //         pair.1.import_data(&self, None);
    //     }
    //     Ok(())
    // }
}

pub fn start() -> Result<()> {
    let manager = SqliteConnectionManager::file(":memory:");
    let db = Db {
        pool: Pool::new(manager)?,
    };

    // add some data providers (e.g. chat-db data from a ship)
    // let mut data_providers = DataProviders::new();
    // data_providers.insert(String::from("chat-db"), Box::new(ChatDataProvider::new()));
    // data_providers.insert(String::from("chat-db"), providers::chat::import_data);

    let mut _res = generate_schema(&db);
    _res = import_data(&db); //, &data_providers);

    return Ok(());
}

fn generate_schema(db: &Db) -> Result<()> {
    // run thru all the sql files in the migrations folder in numerical
    //  order and execute them
    println!("{:?}", env::current_dir());
    let mut paths: Vec<_> = fs::read_dir("src/os/src/modules/db/sql")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());
    let mut rollback: bool = false;
    if db.get_conn().execute_batch("BEGIN TRANSACTION;").is_err() {
        bail!("db: [invoke_genesis] failed to start transaction.");
    }
    for path in paths {
        println!("processing sql file '{}'...", path.path().display());
        // read file contents and execute contents
        let sql = fs::read_to_string(path.path())?;
        if db.get_conn().execute_batch(sql.as_str()).is_err() {
            rollback = true;
            println!(
                "db: [invoke_genesis] failed to run query batch in file {}.",
                path.path().display()
            );
            break;
        }
    }
    if rollback {
        if db
            .get_conn()
            .execute_batch("ROLLBACK TRANSACTION;")
            .is_err()
        {
            bail!("db: [invoke_genesis] failed to rollback transaction transaction.");
        }
    } else {
        if db.get_conn().execute_batch("COMMIT TRANSACTION;").is_err() {
            bail!("db: [invoke_genesis] failed to commit transaction transaction.");
        }
    }
    Ok(())
}

fn import_data(db: &Db) -> Result<()> {
    let _res = providers::chat::import_data(db);
    Ok(())
}
