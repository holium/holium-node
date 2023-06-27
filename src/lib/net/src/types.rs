use lazy_static::lazy_static;
use serde_json::Value;

use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

// use thiserror::Error;

lazy_static! {
  // pub static ref ROOM_MAP: RwLock<HashMap<Rid, RoomTuple>> = RwLock::new(HashMap::new());
  // pub static ref ROOM_MAP: RwLock<HashMap<Rid, RoomLock>> = RwLock::new(HashMap::new());
  // pub static ref PEER_MAP: RwLock<HashMap<PeerId, PeerInfo>> = RwLock::new(HashMap::new());
}

pub type Path = String;

// // pub struct Sub {}
pub trait WebSocketSub: std::fmt::Debug + Clone + Send + Sync {
    fn on_message(&self, sender: UnboundedSender<Message>, message: Value) -> DispatcherResult;
}

pub type WebSocketMessageHandler =
    fn(sender: UnboundedSender<Message>, message: Value) -> Option<()>;

// in case of either sucess or failure, the result will contain a serde_json::Value
pub type DispatcherResult = std::result::Result<Value, Value>;
// in case of either sucess or failure, the result will contain a serde_json::Value
// pub type MessageHandlerResult = std::result::Result<Value, Value>;

// #[derive(Error, Debug)]
// pub enum MessageHandlerError {
//     #[error("Unknown error")]
//     Unknown,
// }
