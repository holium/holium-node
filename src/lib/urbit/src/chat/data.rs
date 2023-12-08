use crate::db::Db;
use anyhow::Result;

use super::types::ChatMessage;

pub async fn query_messages(db: &Db, timestamp: i64) -> Result<Vec<ChatMessage>> {
    let conn = db.pool.get_conn()?;

    let mut stmt = conn.prepare(
        "SELECT path,
                msg_id,
                msg_part_id,
                content_type,
                content_data,
                reply_to,
                metadata,
                sender,
                created_at,
                updated_at,
                received_at,
                expires_at
              FROM chat_messages
              WHERE received_at >= ?1",
    )?;

    let msg_iter = stmt.query_map([timestamp], |row| {
        Ok(ChatMessage {
            path: row.get(0)?,
            msg_id: row.get(1)?,
            msg_part_id: row.get(2)?,
            content_type: row.get(3)?,
            content_data: row.get(4)?,
            reply_to: row.get(5)?,
            metadata: row.get(6)?,
            sender: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            received_at: row.get(10)?,
            expires_at: row.get(11)?,
        })
    })?;

    let mut records: Vec<ChatMessage> = Vec::new();
    for msg in msg_iter {
        records.push(msg?);
    }

    Ok(records)
}
