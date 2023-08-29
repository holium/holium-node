use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

pub type PeerId = String;
pub type PeerIp = String;
pub type Rid = String;

// 4th element:
// minimal data structure to enforce rule that user only be allowed
// in one 'interactive' and/or one 'background' session at a time
// slot 0 is reserved for the current 'interactive' session (if exists)
// slot 1 is reserved for the current 'background' session (if exists)
pub type PeerInfo = (PeerIp, UnboundedSender<Message>, Peer);
pub type Peers = HashMap<PeerId, PeerInfo>;

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
    pub static ref PEER_MAP: RwLock<HashMap<PeerId, PeerInfo>> = RwLock::new(HashMap::new());
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
