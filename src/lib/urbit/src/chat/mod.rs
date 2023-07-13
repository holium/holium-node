use std::{env, fs};

use crate::CallContext;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize, Serialize)]
struct ReplyTo {
    #[serde(rename = "msg-id")]
    msg_id: String,
    path: String,
}

/// derived from chat-db json format
// {
//   "tables": {
//     "messages": [
//       {
//         "received-at": 1688763457727,
//         "metadata": {},
//         "reply-to": {
//           "path": "/spaces/~lomder-librun/realm-forerunners/chats/0v2.68end.ets6m.29fgc.ntejl.jbeo7",
//           "msg-id": "/~2023.7.7..19.57.10..a94a/~fasnut-famden"
//         },
//         "updated-at": 1688763453954,
//         "msg-part-id": 0,
//         "created-at": 1688763453954,
//         "path": "/spaces/~lomder-librun/realm-forerunners/chats/0v2.68end.ets6m.29fgc.ntejl.jbeo7",
//         "content-data": "2764-fe0f",
//         "expires-at": null,
//         "sender": "~tolwer-mogmer",
//         "content-type": "react",
//         "msg-id": "/~2023.7.7..20.57.33..f44a/~tolwer-mogmer"
//       }
//     ]
//   }
// }
#[derive(Debug, Deserialize, Serialize)]
struct ChatMessage {
    #[serde(rename = "msg-id")]
    msg_id: String,
    #[serde(rename = "msg-part-id")]
    msg_part_id: u64,
    path: String,
    metadata: JsonValue,
    sender: String,
    #[serde(rename = "reply-to")]
    reply_to: JsonValue,
    #[serde(rename = "content-type")]
    content_type: String,
    #[serde(rename = "content-data")]
    content_data: String,
    #[serde(rename = "created-at")]
    created_at: u64,
    #[serde(rename = "received-at")]
    received_at: u64,
    #[serde(rename = "updated-at")]
    updated_at: u64,
    #[serde(rename = "expires-at")]
    expires_at: JsonValue, // can be null
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatTables {
    messages: Vec<ChatMessage>,
}

pub async fn generate_schema(ctx: &CallContext) -> Result<()> {
    // run thru all the sql files in the migrations folder in numerical
    //  order and execute them
    println!("{:?}", env::current_dir());
    let mut paths: Vec<_> = fs::read_dir("sql").unwrap().map(|r| r.unwrap()).collect();
    paths.sort_by_key(|dir| dir.path());
    // let conn = mgr.as_ref().unwrap().get().unwrap();
    ctx.db.get_conn()?.execute_batch("BEGIN TRANSACTION")?;
    for path in paths {
        println!("processing sql file '{}'...", path.path().display());
        // read file contents and execute contents
        let sql = fs::read_to_string(path.path())?;
        ctx.db.get_conn()?.execute_batch(sql.as_str())?;
    }
    ctx.db.get_conn()?.execute_batch("COMMIT TRANSACTION")?;
    Ok(())
}

pub async fn import_data(ctx: &CallContext) -> Result<()> {
    // grab a connection from the connection pool
    let conn = ctx.db.get_conn()?;

    // retrieve the last timestamp value from the chat_messages table
    let last_timestamp: Result<i32, _> = conn.query_row(
        "SELECT MAX(received_at) AS last_timestamp FROM chat_messages",
        [],
        |row| row.get(0),
    );

    // scry the ship for chat messages
    let response = ctx
        .ship_interface
        .scry_to_str(
            "chat-db",
            format!("/db/messages/start-ms/{}", last_timestamp?).as_str(),
            "json",
        )
        .await?;

    let tables: ChatTables = serde_json::from_str(&response)?;

    let mut result = ctx.db.get_conn()?.execute_batch("BEGIN");

    if result.is_err() {
        bail!("chat: error in load method. execute_batch failed to start transaction");
    }

    for msg in tables.messages {
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
            expires_at,
            received_at
          ) VALUES (
            :path,
            :msg_id,
            :msg_part_id,
            :content_type,
            :content_data,
            :reply_to,
            :metadata,
            :sender,
            :created_at,
            :updated_at,
            :expires_at,
            :received_at
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
            msg.expires_at,
            msg.received_at,
        ))?;
    }

    result = conn.execute_batch("COMMIT");

    if result.is_err() {
        bail!("chat: error in load method. execute_batch failed to start transaction");
    }

    Ok(())
}

///
/// 1) generate the chat schema
/// 2) import data from ship into the database
///
pub async fn load(ctx: &CallContext) -> Result<()> {
    // ensure latest schema by running scripts in ./sql folder
    generate_schema(ctx).await?;
    // scry ship for latest chat data and add to database
    import_data(ctx).await?;
    Ok(())
}
