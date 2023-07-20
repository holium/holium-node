use anyhow::Result;
use bedrock_db::DbPool;
use serde_json::Value as JsonValue;
use std::time::SystemTime;

// pub trait DbApi {
//     fn save_packet(&self, packet: &JsonValue) -> Result<()>;
// }

#[derive(Debug)]
pub struct Db {
    pub pool: DbPool,
}

impl Db {
    pub fn save_packet(&self, packet: &JsonValue) -> Result<()> {
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let ts: u128 = ts.as_micros();
        let conn = self.pool.get_conn()?;
        let mut stmt = conn.prepare(
            "INSERT INTO packets (
              content,
              received_at
            ) VALUES (
              ?1,
              ?2
            )",
        )?;
        stmt.execute((packet, ts as i64))?;
        Ok(())
    }
}
