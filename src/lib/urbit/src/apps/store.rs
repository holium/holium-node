use serde::Deserialize;
use serde_json::{Map, Value};

use crate::ShipInterface;

use reqwest::Error as ReqError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppStoreAPIError>;

#[derive(Deserialize)]
struct Response {
    initial: Map<String, Value>,
}

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

// to truly uniquely identify apps on the Urbit network, you need the "fully qualified path";
//  or the a string of the form: "<ship>/<desk>"
pub struct AppURI {
    // name of ship where app was either originally downloaded, or the ship from
    //  which app udpates should be installed. assuming here that the UI will allow for
    //  changing the host ship of any app
    pub ship_name: String,
    // name of the desk (e.g. realm, base, garden, landscape, et. al.)
    pub desk: String,
}

#[derive(Debug)]
pub enum AppStatus {
    // active and running
    Running,
    Suspended,
}

#[derive(Debug)]
pub enum UpdateStatus {
    // receive updates via commits on the local ship
    Local,
    // receive updates from the source ship
    Tracking,
    // do not receive updates
    Paused,
}

//
#[derive(Debug)]
pub struct AppListing {
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
    // original publishing ship
    pub publishing_ship: String,
    // updates status
    pub updates: UpdateStatus,
    // the desk on the source ship
    pub source_desk: String,
    // The revision number of the desk on the source ship
    pub source_aeon: String,
    // Updates waiting to be applied due to incompatibility
    pub pending_updates: Vec<String>,
    // docket info
}

// #[derive(Deserialize, Debug)]
// struct Docket {
//     desk: String,
//     id: u32,
// }

// combine scries across various agents to create one unified payload capable of driving the entire
//  Realm desktop grid
pub async fn get_app_detail(
    ship_interface: ShipInterface,
    desk: String,
) -> Result<Map<String, Value>> {
    let docket_res = ship_interface
        .scry("docket", "/charges", "json")
        .await
        .unwrap();
    let jon: Value = serde_json::from_str(&docket_res.text().await.unwrap()).unwrap();
    // the response comes in as an "initial" payload. rather than include that noise
    //  in our struct, leverage a custom serializer to get a
    let map: Map<String, Value> = jon.as_object().unwrap().clone();
    if !map.contains_key("initial") {
        return Err(AppStoreAPIError::AppNotFound);
    }
    let mut desk: Option<Map<String, Value>> = None;
    let desks: Map<String, Value> = serde_json::from_value(*map.get("initial").unwrap()).unwrap();
    if !desks.contains_key(desk) {
        desk = serde_json::from_value(*map.get(desk).unwrap()).unwrap();
    }
    return Ok(desk);
    //let users: Vec<Docket> = response.json::<Docket>().await?;
    // println!("{:?}", users);
}
