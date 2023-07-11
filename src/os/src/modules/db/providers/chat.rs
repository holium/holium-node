use crate::modules::db::Db;
use std::format;

use serde::{Deserialize, Serialize};

use super::ship::Ship;

use rusqlite::named_params;

use anyhow::{bail, Result};

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    #[serde(rename = "msg-id")]
    msg_id: String,
    #[serde(rename = "content-type")]
    content_type: String,
    sender: String,
    path: String,
    #[serde(rename = "content-data")]
    content_data: String,
    #[serde(rename = "received-at")]
    received_at: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tables {
    messages: Vec<Message>,
}

// pub struct ChatDataProvider {
//     // req: Option<Client>,
//     options: Option<HashMap<String, String>>,
//     auth_token: Option<String>,
// }

// impl ChatDataProvider {
//     pub fn new() -> Self {
//         return ChatDataProvider {
//             req: None,
//             options: None,
//             auth_token: None,
//         };
//     }

//     async fn login(&self, url: &str, ship_code: &str) -> Result<String> {
//         let login_url = format!("{}/~/login", url);
//         let resp = self
//             .req
//             .as_ref()
//             .unwrap()
//             .post(&login_url)
//             .body("password=".to_string() + ship_code)
//             .send()
//             .await?;
//         // Check for status code
//         if resp.status().as_u16() != 204 {
//             bail!("chat_data_provider: login failed");
//         }
//         if !resp.headers().contains_key("set-cookie") {
//             bail!("chat_data_provider: login failed. cookie not found");
//         }
//         let auth_token = resp.headers().get("set-cookie").unwrap().to_str();
//         if auth_token.is_err() {
//             bail!("chat_data_provider: login failed. cookie failure");
//         }
//         return Ok(auth_token.unwrap().to_string());
//     }

//     /// Sends a scry to the ship
//     async fn scry(&self, scry_url: &str) -> Result<String> {
//         let resp = self
//             .req
//             .as_ref()
//             .unwrap()
//             .get(scry_url)
//             .header(COOKIE, self.auth_token.clone().unwrap())
//             .header("Content-Type", "application/json");
//         let result = resp.send().await?;
//         if result.status().as_u16() != 200 {
//             bail!(
//                 "chat_data_provider: scry failed {}",
//                 result.status().as_u16()
//             );
//         }
//         Ok(result.text().await?)
//     }
// }

// #[async_trait]
// impl DataProvider for ChatDataProvider {
//     async fn import_data(&self, db: &Db, _args: Option<HashMap<&str, String>>) -> Result<()> {
//         Ok(())
//     }
// }

pub async fn import_data(db: &Db) -> Result<()> {
    // retrieve the last timestamp value from the chat_messages table
    let last_timestamp: Result<i32, _> = db.get_conn().query_row(
        "SELECT MAX(received_at) AS last_timestamp FROM chat_messages",
        [],
        |row| row.get(0),
    );

    let mut ship = Ship::new("http://localhost:9030", "napdem-fopbex-mapbus-ridmel");

    let result = ship.login().await;

    if result.is_err() {
        bail!(format!("{}", result.err().unwrap().to_string()));
    }

    let response = ship
        .scry(
            "chat-db",
            &format!("/db/messages/start-ms/{}", last_timestamp?),
            "json",
        )
        .await;

    if response.is_err() {
        bail!(format!("{}", response.err().unwrap().to_string()));
    }

    let tables: Tables = serde_json::from_str(&response.unwrap())?;
    for message in tables.messages {
        let _res = db.get_conn().execute(
            "INSERT INTO chat_messages(
            msg_id,
            content_type,
            sender,
            path,
            content_data,
            received_at
          ) VALUES (
            :msg_id,
            :content_type,
            :sender,
            :path,
            :content_data,
            :received_at
          )",
            named_params! {
                ":msg_id": message.msg_id,
                ":content_type": message.content_type,
                ":sender": message.sender,
                ":path": message.path,
                ":content_data": message.content_data,
                ":received_at": message.received_at
            },
        );
    }

    Ok(())
}
