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
struct ChatTable {
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatTables {
    tables: ChatTable,
}

pub async fn generate_schema(ctx: &CallContext) -> Result<()> {
    // run thru all the sql files in the migrations folder in numerical
    //  order and execute them
    println!("current directory: {:?}", env::current_dir().unwrap());
    let mut paths: Vec<_> = fs::read_dir("src/lib/urbit/src/chat/sql")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
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
    let last_timestamp: Result<i64, _> = conn.query_row(
        "SELECT COALESCE(MAX(received_at), 0) AS last_timestamp FROM chat_messages",
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

    println!("deserializing chat messages retrieved from ship...");

    let root: ChatTables = serde_json::from_str(&response)?;

    println!("processing chat messages...");

    let mut result = conn.execute_batch("BEGIN");

    if result.is_err() {
        bail!("chat: error in load method. execute_batch failed to start transaction");
    }

    for msg in root.tables.messages {
        println!("processing chat message: {:?}", msg);
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
    Ok(())
}
