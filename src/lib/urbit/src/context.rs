use crate::SafeShipInterface;
use bedrock_db::db::Db;

pub struct CallContext {
    pub db: Db,
    pub ship_interface: SafeShipInterface,
}