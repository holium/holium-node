use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyTo {
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
pub struct ChatMessage {
    #[serde(rename = "msg-id")]
    pub msg_id: String,
    #[serde(rename = "msg-part-id")]
    pub msg_part_id: u64,
    pub path: String,
    pub metadata: JsonValue,
    pub sender: String,
    #[serde(rename = "reply-to")]
    pub reply_to: JsonValue,
    #[serde(rename = "content-type")]
    pub content_type: String,
    #[serde(rename = "content-data")]
    pub content_data: String,
    #[serde(rename = "created-at")]
    pub created_at: u64,
    #[serde(rename = "received-at")]
    pub received_at: u64,
    #[serde(rename = "updated-at")]
    pub updated_at: u64,
    #[serde(rename = "expires-at")]
    pub expires_at: JsonValue, // can be null
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatTable {
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatTables {
    pub tables: ChatTable,
}
