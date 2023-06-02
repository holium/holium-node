use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::ShipInterface;

use reqwest::Error as ReqError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppStoreAPIError>;

// #[derive(Deserialize)]
// struct Response {
//     initial: Map<String, Value>,
// }

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

// // to truly uniquely identify apps on the Urbit network, you need the "fully qualified path";
// //  or the a string of the form: "<ship>/<desk>"
// pub struct AppURI {
//     // name of ship where app was either originally downloaded, or the ship from
//     //  which app udpates should be installed. assuming here that the UI will allow for
//     //  changing the host ship of any app
//     pub ship_name: String,
//     // name of the desk (e.g. realm, base, garden, landscape, et. al.)
//     pub desk: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppStatus {
    // active and running
    Running,
    Suspended,
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStatus {
    // receive updates via commits on the local ship
    Local,
    // receive updates from the source ship
    Tracking,
    // do not receive updates
    Paused,
    #[default]
    Unknown,
}

//
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDetail {
    // Universal resource identifier as a string of the form: '<ship>/<desk>'
    pub uri: String,
    // Ship name (parsed from uri)
    pub ship_name: String,
    // Desk name (parsed from uri)
    pub app_name: String,
    // Kelvin supported versions
    pub sys_kelvin: Vec<String>,
    // base hash ends in
    pub base_hash: String,
    // %cz hash ends in
    pub cz_hash: String,
    // app status as reported by +vats interface
    pub app_status: AppStatus,
    // // original publishing ship
    // pub publishing_ship: String,
    // // updates status
    // pub updates: UpdateStatus,
    // // the desk on the source ship
    // pub source_desk: String,
    // // The revision number of the desk on the source ship
    // pub source_aeon: String,
    // Updates waiting to be applied due to incompatibility
    pub pending_updates: Vec<String>,
    //
    // docket info
    //
    // pub image: String,
    // pub title: String,
    // pub license: String,
    // pub version: String,
    // pub website: String,
    // pub href: Href,
    // pub chad: Chad,
    // pub color: String,
    // pub info: String,
}

pub async fn get_apps(ship_interface: ShipInterface) -> Result<Map<String, Value>> {
    let apps_res = ship_interface.scry("holon", "/apps", "json").await.unwrap();
    let jon: Value = serde_json::from_str(&apps_res.text().await.unwrap()).unwrap();
    // the response comes in as an "initial" payload. rather than include that noise
    //  in our struct, leverage a custom serializer to get a
    let map: Map<String, Value> = jon.as_object().unwrap().clone();
    if !map.contains_key("initial") {
        return Err(AppStoreAPIError::MissingAppData);
    }
    let initial: Value = serde_json::to_value(map.get("initial").unwrap()).unwrap();
    let desks: Map<String, Value> = serde_json::from_value(initial).unwrap();

    return Ok(desks);
}

// combine scries across various agents to create one unified payload capable of driving the entire
//  Realm desktop grid
pub async fn get_app_detail(
    ship_interface: ShipInterface,
    desk: &str,
) -> Result<Map<String, Value>> {
    let path: String = format!("/apps/{desk}");
    let app_res = ship_interface.scry("holon", &path, "json").await.unwrap();
    let jon: Value = serde_json::from_str(&app_res.text().await.unwrap()).unwrap();
    // the response comes in as an "initial" payload. rather than include that noise
    //  in our struct, leverage a custom serializer to get a
    let map: Map<String, Value> = jon.as_object().unwrap().clone();
    if !map.contains_key("initial") {
        return Err(AppStoreAPIError::MissingAppData);
    }
    let initial: Value = serde_json::to_value(map.get("initial").unwrap()).unwrap();
    let desks: Map<String, Value> = serde_json::from_value(initial).unwrap();
    if !desks.contains_key(desk) {
        return Err(AppStoreAPIError::AppNotFound);
    }
    // borrow the desk from the desks map
    let app: Value = serde_json::to_value(desks.get(desk).unwrap()).unwrap();
    return Ok(serde_json::from_value(app).unwrap());
}
