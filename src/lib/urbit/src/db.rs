use anyhow::Result;
use bedrock_db::db::Db;
use serde_json::Value as JsonValue;
use std::time::SystemTime;

pub fn save_packet(db: &Db, packet: &JsonValue) -> Result<()> {
    save_packet_string(db, packet.to_string())
}

pub fn save_packet_string(db: &Db, packet: String) -> Result<()> {
    let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let ts: u128 = ts.as_micros();
    let conn = db.get_conn()?;
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
