use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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

pub async fn get_apps(ship_interface: ShipInterface) -> Result<Map<String, Value>> {
    let apps_res = ship_interface
        .scry("app-store", "/apps", "json")
        .await
        .unwrap();
    return Ok(apps_res.json().await.unwrap());
}

// combine scries across various agents to create one unified payload capable of driving the entire
//  Realm desktop grid
pub async fn get_app_detail(
    ship_interface: ShipInterface,
    desk: &str,
) -> Result<Map<String, Value>> {
    let path: String = format!("/apps/{desk}");
    let app_res = ship_interface
        .scry("app-store", &path, "json")
        .await
        .unwrap();
    return Ok(app_res.json().await.unwrap());
}
