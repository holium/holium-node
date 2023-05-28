use lazy_static::lazy_static;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

pub type PeerId = String;
pub type PeerIp = String;
pub type Rid = String;

pub type PeerInfo = (PeerIp, UnboundedSender<Message>, Peer);
pub type Peers = HashMap<PeerId, PeerInfo>;

pub type PeerMap = Arc<RwLock<Peers>>;
pub type PeerIds = Arc<RwLock<VecDeque<PeerId>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: PeerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub rid: String,
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

pub type RoomTuple = (PeerMap, RoomLock);

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