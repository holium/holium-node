#[cfg(feature = "trace")]
use std::env;
use std::fs;

use crate::context::CallContext;
use anyhow::{bail, Result};

use super::types::ChatTables;
use trace::trace_info_ln;

pub async fn generate_schema(ctx: &CallContext) -> Result<()> {
    // run thru all the sql files in the migrations folder in numerical
    //  order and execute them
    trace_info_ln!("current directory: {:?}", env::current_dir().unwrap());

    let mut paths: Vec<_> = fs::read_dir("src/lib/urbit/src/chat/sql")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());
    // let conn = mgr.as_ref().unwrap().get().unwrap();
    ctx.db.pool.get_conn()?.execute_batch("BEGIN TRANSACTION")?;
    for path in paths {
        trace_info_ln!("processing sql file '{}'...", path.path().display());
        // read file contents and execute contents
        let sql = fs::read_to_string(path.path())?;
        ctx.db.pool.get_conn()?.execute_batch(sql.as_str())?;
    }
    ctx.db
        .pool
        .get_conn()?
        .execute_batch("COMMIT TRANSACTION")?;
    Ok(())
}

pub async fn import_data(ctx: &CallContext) -> Result<()> {
    // grab a connection from the connection pool
    let conn = ctx.db.pool.get_conn()?;

    // retrieve the last timestamp value from the chat_messages table
    let last_timestamp: Result<i64, _> = conn.query_row(
        "SELECT COALESCE(MAX(received_at), 0) AS last_timestamp FROM chat_messages",
        [],
        |row| row.get(0),
    );

    // scry the ship for chat messages
    let response = ctx
        .ship
        .lock()
        .await
        .scry(
            "chat-db",
            format!("/db/messages/start-ms/{}", last_timestamp?).as_str(),
            "json",
        )
        .await?;

    // println!("deserializing chat messages retrieved from ship...");

    let root: ChatTables = serde_json::from_value(response)?;

    trace_info_ln!("processing chat messages...");

    let mut result = conn.execute_batch("BEGIN");

    if result.is_err() {
        bail!("chat: error in load method. execute_batch failed to start transaction");
    }

    for msg in root.tables.messages {
        // println!("processing chat message: {:?}", msg);
        let mut stmt = conn.prepare(
            "REPLACE INTO chat_messages (
                    path,
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
                  ) VALUES (
                    ?1,
                    ?2,
                    ?3,
                    ?4,
                    ?5,
                    ?6,
                    ?7,
                    ?8,
                    ?9,
                    ?10,
                    ?11,
                    ?12
                  )",
        )?;
        stmt.execute((
            msg.path,
            msg.msg_id,
            msg.msg_part_id,
            msg.content_type,
            msg.content_data,
            msg.reply_to,
            msg.metadata,
            msg.sender,
            msg.created_at,
            msg.updated_at,
            msg.received_at,
            msg.expires_at,
        ))?;
    }

    result = conn.execute_batch("COMMIT");

    if result.is_err() {
        bail!("chat: error in load method. execute_batch failed to commit transaction");
    }

    Ok(())
}

///
/// 1) generate the chat schema
/// 2) import data from ship into the database
///
pub async fn start(ctx: &CallContext) -> Result<()> {
    // ensure latest schema by running scripts in ./sql folder
    generate_schema(ctx).await?;
    // scry ship for latest chat data and add to database
    import_data(ctx).await?;
    // super::sub::listen(ctx);
    Ok(())
}
