use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

pub type PeerId = String;
pub type DeviceId = String;
pub type PeerIp = String;
pub type Rid = String;

// 4th element:
// minimal data structure to enforce rule that user only be allowed
// in one 'interactive' and/or one 'background' session at a time
// slot 0 is reserved for the current 'interactive' session (if exists)
// slot 1 is reserved for the current 'background' session (if exists)
pub type DeviceInfo = (PeerIp, UnboundedSender<Message>, Peer);
// pub type Peers = HashMap<PeerId, PeerInfo>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Background,
    Interactive,
}

impl RoomType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoomType::Background => "background",
            RoomType::Interactive => "interactive",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: PeerId,
    // pub rooms: Arc<RwLock<[Option<()>; 2]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub rid: String,
    // room type:
    //  "interactive" | "background"
    pub rtype: String,
    pub title: String,
    pub creator: String,
    pub provider: String,
    pub access: String,
    pub present: Vec<String>,
    pub whitelist: Vec<String>,
    pub capacity: u32,
    pub path: Option<String>,
}
pub type RoomLock = Arc<RwLock<Room>>;

lazy_static! {
    // pub static ref ROOM_MAP: RwLock<HashMap<Rid, RoomTuple>> = RwLock::new(HashMap::new());
    pub static ref ROOM_MAP: RwLock<HashMap<Rid, RoomLock>> = RwLock::new(HashMap::new());
    // @patrick - thought about creating a unique string key of "peer_id:peer_ip"; however
    //  sending messages back out requires string parsing and loops to determine all the sockets to send on.
    //  a map of devices should be more efficient since lookups will be faster.
    // also .. choosing to use a Vector of devices instead of a map since it is possible for multiple realm
    //   electron clients (running on the same device) to have the same IP address.
    // we can either force devs to come up with a truly unique device ID (will be difficult to be 100% fool proof
    //   for electron clients), or simply allow the same IP to connect multiple times (if needed)
    pub static ref PEER_MAP: RwLock<HashMap<PeerId, HashMap<DeviceId, DeviceInfo>>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Ok {
        rtype: String,
    },
    Error {
        rtype: String,
        message: String,
    },
    RoomUpdate {
        rtype: String,
        room: String,
        peers: Vec<Peer>,
    },
    PeerJoined {
        rtype: String,
        peer_id: PeerId,
    },
    PeerLeft {
        rtype: String,
        peer_id: PeerId,
    },
}
