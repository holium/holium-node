use crate::SafeShipInterface;
use bedrock_db::db::Db;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct CallContext {
    pub db: Db,
    pub ship_interface: SafeShipInterface,
    pub sender: UnboundedSender<JsonValue>,
    pub receiver: UnboundedReceiver<JsonValue>,
}
