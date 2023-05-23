use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ChatRow {
    id: u32,
    app: String,
    path: String,
    #[serde(rename = "type")]
    type_: String,
    title: String,
    content: String,
    image: String,
    buttons: Value, // in Rust, we could use serde_json::Value to represent an arbitrary serialized JSON data
    link: String,
    metadata: Value, // same as above
    created_at: u64,
    updated_at: u64,
    read_at: Option<u64>,
    read: bool,
    dismissed_at: Option<u64>,
    dismissed: bool,
}

// // Define an enum `DbChangeType` to represent the different variants of changes.
// pub enum DbChangeType {
//     AddRow(DbRow),
//     UpdMessages(MsgId, Message),
//     UpdPathsRow(PathRow, PathRow),
//     DelPathsRow(Path, DateTime<Utc>),
//     DelPeersRow(Path, Ship, DateTime<Utc>),
//     DelMessagesRow(Path, UniqId, DateTime<Utc>),
// }

// // Define `DbRow` enum to hold the different types of rows
// pub enum DbRow {
//     Paths(PathRow),
//     Messages(MsgPart),
//     Peers(PeerRow),
// }

// // `DbChange` is simply a vector of `DbChangeType`
// pub type DbChange = Vec<DbChangeType>;

// // `DelLog` is a vector of tuples, where each tuple contains a DateTime and a DbChangeType
// pub type DelLog = Vec<(DateTime<Utc>, DbChangeType)>;
