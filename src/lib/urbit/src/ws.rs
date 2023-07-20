// #![deny(warnings)]
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::SystemTime;

use anyhow::Result;

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::context::CallContext;

/// Our global unique device id counter.
static NEXT_DEVICE_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected devices.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type DeviceMap = HashMap<usize, mpsc::UnboundedSender<Message>>;
type Devices = Arc<RwLock<DeviceMap>>;

lazy_static! {
    static ref CONNECTED_DEVICES: Devices = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ShipAction {
    pub id: u64,
    pub action: String,
    pub ship: String,
    pub app: String,
    pub mark: String,
    pub json: JsonValue,
}

pub fn start(
    context: CallContext,
    // ship_proxy: Ship,
    // receiver: UnboundedReceiver<JsonValue>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Turn our "state" into a new Filter...

    // let devices = Devices::default();
    // let devices = warp::any().map(move || devices.clone());

    // let with_ship_proxy = warp::any().map(move || ship_proxy.clone());

    // let receiver = Arc::new(Mutex::new(receiver));
    // let ship_event_receiver = warp::any().map(move || receiver.clone());

    let with_context = warp::any().map(move || context.clone());

    // GET /chat -> websocket upgrade
    let handler = warp::path!("ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        // .and(devices)
        // .and(with_ship_proxy)
        // .and(ship_event_receiver)
        .and(with_context)
        .map(
            |ws: warp::ws::Ws,
             context: CallContext /*devices, ship_proxy: Ship,
                                  ship_event_receiver: ShipReceiver*/| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| {
                    device_connected(
                        socket,
                        context.clone(), /*ship_proxy, ship_event_receiver*/
                    )
                })
            },
        );

    handler
}

async fn device_connected(
    ws: WebSocket,
    context: CallContext, /*ship_event_receiver: ShipReceiver*/
) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_DEVICE_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut device_ws_tx, mut device_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    // spawn a task to listen for messages to send to transmit to connected devices
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            device_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected devices.
    CONNECTED_DEVICES.write().await.insert(my_id, tx);

    let device_ws_rx_context = context.clone();

    // now spawn a task to listen for incoming messages from connected devices
    tokio::task::spawn(async move {
        // Every time the device sends a message, broadcast it to
        // all other devices...
        while let Some(result) = device_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket error(uid={}): {}", my_id, e);
                    break;
                }
            };
            // load a handler for the message
            on_device_message(my_id, msg, &device_ws_rx_context).await;
        }
    });

    /////////////////////
    //
    // SHIP update/event listener
    //  mpsc (multi-producer single consumer) means that there should only ever be one
    //   ship listening receiver; therefore safe to lock (i.e. no race conditions or other blocking concerns
    println!("ws: [device_connected] waiting for ship event...");
    let mut receiver = context.receiver.lock().await;
    while let Some(result) = receiver.recv().await {
        println!(
            "ws: [device_connected] received event from ship => [{}, {}]",
            my_id, result
        );
        on_ship_message(my_id, result).await;
    }
    //
    /////////////////////

    // device_ws_rx stream will keep processing as long as the device stays
    // connected. Once they disconnect, then...
    on_device_disconnected(my_id).await;
}

///
async fn on_device_message(my_id: usize, msg: Message, context: &CallContext) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    println!("ws: [device_message] [{}, {}]", my_id, msg);

    // let data: serde_json::Result<Vec<ShipAction>> = serde_json::from_str(msg);
    // let data: serde_json::Result<Vec<ShipAction>> = serde_json::from_str(msg);
    // if data.is_err() {
    //     println!("ws: [device_message] payload not valid json");
    //     return;
    // }

    let packet: serde_json::Result<JsonValue> = serde_json::from_str(msg);

    if packet.is_err() {
        println!("ws: [device_message] payload not valid json");
        return;
    }

    let packet = packet.unwrap();

    // 1) save packet payload to db
    let result = save_packet(&context, &packet).await;
    if result.is_err() {
        println!("ws: [device_message] save_packet_string failed");
        return;
    }

    // 2) post action payload to ship. event source receiver will relay any updates/effects
    //     back to connected devices
    let result = { context.ship.post(&packet).await };

    if result.is_err() {
        println!("ws: [device_message] proxy.post call failed");
        return;
    }

    // send the proxy post response back to the originating device over websocket
    let tx = CONNECTED_DEVICES.read();
    let tx = tx.await;
    let tx = tx.get(&my_id);
    {
        if tx.is_none() {
            println!("ws: [device_message] error attempting to read device {} from list of connected devices", my_id);
            return;
        }
        let tx = tx.unwrap();
        let _ = tx.send(Message::text(msg.clone()));
    }

    // deserialize string into json array (Vec) of Poke structures
    // let packets: Vec<ShipAction> = data.unwrap();
    // for packet in packets {
    //     let record = serde_json::to_value(packet);
    //     if record.is_err() {
    //         println!("ws: [device_message] unexpected result for serde_json::to_value call");
    //         continue;
    //     }
    //     let result = crate::db::save_packet(record.unwrap());
    //     if result.is_err() {
    //         println!("ws: [device_message] db::save_packet call failed");
    //         continue;
    //     }
    // }

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in CONNECTED_DEVICES.read().await.iter() {
        if my_id == uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn on_ship_message(my_id: usize, msg: JsonValue) {
    // New message from the ship, send it to all connected devices (except same uid)...
    for (&uid, tx) in CONNECTED_DEVICES.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.as_str().unwrap())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn on_device_disconnected(my_id: usize) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    CONNECTED_DEVICES.write().await.remove(&my_id);
}

async fn save_packet(ctx: &CallContext, packet: &JsonValue) -> Result<()> {
    let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let ts: u128 = ts.as_millis();
    let conn = ctx.db.get_conn()?;
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

// fn with_db(ctx: CallContext) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
//     warp::any().map(move || ctx.db.clone())
// }

// fn with_ship(
//     ctx: CallContext,
// ) -> impl Filter<Extract = (SafeShipInterface,), Error = Infallible> + Clone {
//     warp::any().map(move || ctx.ship_interface.clone())
// }

#[cfg(test)]
mod tests {
    use tungstenite::connect;
    use url::Url;
    // use websocket::ClientBuilder;
    #[test]
    // connect to this node's websocket server (for receiving events from a ship)
    fn can_ws_connect() {
        println!("ws: [test][can_ws_connect] connecting to node websocket...");
        // let client = ClientBuilder::new("ws://127.0.0.1:3030/ws")
        //     .unwrap()
        //     .connect_insecure()
        //     .unwrap();
        let (mut socket, _response) =
            connect(Url::parse("ws://127.0.0.1:3030/ws").unwrap()).expect("Can't connect");
        loop {
            println!("ws: [test][can_ws_connect] waiting for ship events...");
            let msg = socket.read_message().expect("Error reading message");
            println!("Received: {}", msg);
        }
    }
}
