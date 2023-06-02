use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

extern crate bedrock_db;
use crate::ShipInterface;

use reqwest::Error as ReqError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppStoreAPIError>;
pub type AppListing = Map<String, AppDetail>;

#[derive(Error, Debug)]
pub enum AppStoreAPIError {
    // "initial" field not found in /charges scry
    #[error("Missing app data")]
    MissingAppData,
    // desk not found in docket charges
    #[error("App not found")]
    AppNotFound,
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    ReqwestError(#[from] ReqError),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppStatus {
    // active and running
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "suspended")]
    Suspended,
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStatus {
    // receive updates via commits on the local ship
    #[serde(rename = "local")]
    Local,
    // receive updates from the source ship
    #[serde(rename = "remote")]
    Tracking,
    // do not receive updates
    Paused,
    #[default]
    Unknown,
}

pub type Glob = Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Href {
    pub glob: Glob,
    pub base: String,
}

//
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDetail {
    pub status: AppStatus,
    pub image: String,
    #[serde(default)]
    pub kids_desk: Value,
    pub title: String,
    #[serde(default)]
    pub pending_updates: Value,
    pub license: String,
    pub version: String,
    pub publishing_ship: String,
    pub source_desk: String,
    pub sys_kelvin: Vec<i64>,
    pub website: String,
    pub base_hash: String,
    pub desk_hash: String,
    pub href: Option<Href>,
    pub type_: String,
    pub source_ship: String,
    pub updates: UpdateStatus,
    pub source_aeon: String,
    pub color: String,
    pub info: String,
}

pub async fn init(ship_interface: ShipInterface) -> Result<bool> {
    let conn = bedrock_db::conn::establish_connection_pool(":memory:")
        .unwrap()
        .get()
        .unwrap();

    let sql = r#"
        BEGIN;
        CREATE TABLE IF NOT EXISTS app_hosts (
                    path TEXT PRIMARY KEY,
                    ship TEXT NOT NULL,
                    desk TEXT NOT NULL
                );

        INSERT INTO app_hosts (path, ship, desk) VALUES ('~hostyv/realm', '~hostyv', 'realm');
        INSERT INTO app_hosts (path, ship, desk) VALUES ('~paldev/face', '~paldev', 'face');

        CREATE TABLE IF NOT EXISTS app_store (
                    id INTEGER PRIMARY KEY,
                    desk TEXT NOT NULL,
                    title TEXT NOT NULL,
                    status TEXT NOT NULL,
                    image TEXT NOT NULL,
                    kids_desk TEXT NOT NULL,
                    pending_updates TEXT NOT NULL,
                    license TEXT NOT NULL,
                    version TEXT NOT NULL,
                    publishing_ship TEXT NOT NULL,
                    source_desk TEXT NOT NULL,
                    sys_kelvin TEXT NOT NULL,
                    website TEXT NOT NULL,
                    base_hash TEXT NOT NULL,
                    desk_hash TEXT NOT NULL,
                    href TEXT NOT NULL,
                    type_ TEXT NOT NULL,
                    source_ship TEXT NOT NULL,
                    updates TEXT NOT NULL,
                    source_aeon TEXT NOT NULL,
                    color TEXT NOT NULL,
                    info TEXT NOT NULL
                );

        COMMIT;
    "#;

    match conn.execute_batch(sql) {
        Ok(_) => (),
        Err(error) => return Err(AppStoreAPIError::Other(error.to_string())),
    };

    // for now, load app hosts and app globs from the apps that exist on our ship
    match get_apps(ship_interface).await {
        Ok(apps) => {
            let mut sql: String = String::new().to_owned();
            for (desk, app) in apps {
                match app["type"] {
                    Value::String(ref type_) => {
                        if type_ != "urbit" {
                            continue;
                        }
                    }
                    _ => continue,
                }
                let source_ship: Value = match &app["source_ship"] {
                    Value::Null => Value::Null,
                    _ => ().into(),
                };
                let source_ship = match source_ship {
                    Value::Null => {
                        let source_ship: String = match &app["publishing_ship"] {
                            Value::Null => continue,
                            val => val.to_string(),
                        };
                        source_ship
                    }
                    val => val.to_string(),
                };
                let ship = source_ship;
                let path = format!("/{ship}/{desk}");
                sql.push_str(&format!(
                    "INSERT INTO app_hosts (path, ship, desk) VALUES ('{}', '{}', '{}')",
                    path, ship, desk
                ));
            }
            match conn.execute_batch(&sql) {
                Ok(_) => (),
                Err(error) => return Err(AppStoreAPIError::Other(error.to_string())),
            };
        }
        Err(error) => return Err(error),
    };

    return Ok(true);
}

pub async fn get_apps(ship_interface: ShipInterface) -> Result<Map<String, Value>> {
    match ship_interface.scry("app-store", "/apps", "json").await {
        Ok(apps_res) => return Ok(apps_res.json().await.unwrap()),
        Err(error) => return Err(AppStoreAPIError::Other(error.to_string())),
    };
}

pub async fn get_app_detail(
    ship_interface: ShipInterface,
    desk: &str,
) -> Result<Map<String, Value>> {
    let path: String = format!("/apps/{desk}");
    match ship_interface.scry("app-store", &path, "json").await {
        Ok(app_res) => return Ok(app_res.json().await.unwrap()),
        Err(error) => return Err(AppStoreAPIError::Other(error.to_string())),
    }
}
