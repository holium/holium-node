use serde_json::Value;
use warp::ws::Message;

use tokio::sync::mpsc::UnboundedSender;

use crate::SafeShipInterface;
// use net::types::WebSocketMessageHandler;

// all things Chat
pub struct ChatHub {
    pub db: bedrock_db::db::Db,
    pub ship_interface: SafeShipInterface,
    // pub message_handler: WebSocketMessageHandler,
}

impl ChatHub {
    pub fn new(db: bedrock_db::db::Db, ship_interface: SafeShipInterface) -> Self {
        ChatHub { db, ship_interface }
    }

    pub fn handle_message(&self, sender: UnboundedSender<Message>, message: Value) -> Option<()> {
        // let data = message.as_object().unwrap();
        // let message = json!({
        //     "type": "signal",
        //     "rid": rid.clone(),
        //     "from": from,
        //     "signal": signal,
        // });
        // sender.send(Message::text(message.to_string())).unwrap();
        // match data["type"].as_str().unwrap() {
        //     "add-row" => {}
        //     _ => println!("[dispatcher] unknown message type"),
        // }
        Some(())
    }
}

// make ChatHub websocket compatible
// impl Sub for ChatHub {
//     fn on_message(&self, sender: UnboundedSender<Message>, message: Value) -> DispatcherResult {
//         // let data = message.as_object().unwrap();
//         // let message = json!({
//         //     "type": "signal",
//         //     "rid": rid.clone(),
//         //     "from": from,
//         //     "signal": signal,
//         // });
//         // sender.send(Message::text(message.to_string())).unwrap();
//         // match data["type"].as_str().unwrap() {
//         //     "add-row" => {}
//         //     _ => println!("[dispatcher] unknown message type"),
//         // }
//         Ok(json! {
//             {
//                 "ok": true
//             }
//         })
//     }
// }

pub fn init(db: bedrock_db::db::Db, ship_interface: SafeShipInterface) -> Result<ChatHub, ()> {
    let hub: ChatHub = ChatHub::new(db, ship_interface);

    // todo: subscribe to chat channel, pull latest data from ship, etc...
    // ship_interface.scry("chat-view", "json", json!({}))?;

    // hub.db.execute_batch("BEGIN TRANSACTION;");
    // load all chat data since last timestamp
    //  see chat.db.ts in Realm js
    //  add all new messages to chat db
    // hub.db.execute_batch("COMMIT TRANSACTION;");
    return Ok(hub);
}

pub fn handle_message(sender: UnboundedSender<Message>, message: Value) -> Option<()> {
    // let data = message.as_object().unwrap();
    // let message = json!({
    //     "type": "signal",
    //     "rid": rid.clone(),
    //     "from": from,
    //     "signal": signal,
    // });
    // sender.send(Message::text(message.to_string())).unwrap();
    // match data["type"].as_str().unwrap() {
    //     "add-row" => {}
    //     _ => println!("[dispatcher] unknown message type"),
    // }
    Some(())
}
