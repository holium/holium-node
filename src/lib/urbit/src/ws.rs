// #![deny(warnings)]
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
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

// lazy_static! {
//     static ref CONNECTED_DEVICES: Devices = Arc::new(RwLock::new(HashMap::new()));
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct ShipAction {
    pub id: u64,
    pub action: String,
    pub ship: String,
    pub app: String,
    pub mark: String,
    pub json: JsonValue,
}

#[derive(Debug)]
struct MissingAuthToken;

impl warp::reject::Reject for MissingAuthToken {}

pub async fn start(
    context: CallContext,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Turn our "state" into a new Filter...
    let devices = Devices::default();
    let devices = warp::any().map(move || devices.clone());

    let with_context = warp::any().map(move || context.clone());

    // GET /chat -> websocket upgrade
    let handler = warp::path!("ws")
        .and(warp::header::headers_cloned())
        .and(with_context)
        .and(devices)
        .and_then(
            |headers: HeaderMap, context: CallContext, devices: Devices| async move {
                let cookie_key = format!("urbauth-~{:?}", context.ship.lock().await.ship_name);
                if !headers.contains_key("cookie") {
                    return Err(warp::reject::custom(MissingAuthToken));
                }
                println!("ws: [start] token => {:?}", headers.get(cookie_key));
                Ok((context, devices))
            },
        )
        .and(warp::ws())
        .map(
            |(context, devices): (CallContext, Devices), ws: warp::ws::Ws| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| device_connected(socket, devices, context.clone()))
            },
        );

    handler
}

async fn device_connected(
    ws: WebSocket,
    devices: Devices,
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
    devices.write().await.insert(my_id, tx);

    let device_ws_rx_context = context.clone();
    let ws_rx_devices = devices.clone();

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
            on_device_message(my_id, msg, &device_ws_rx_context, &ws_rx_devices).await;
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
        on_ship_message(my_id, result, &devices).await;
    }
    //
    /////////////////////

    // device_ws_rx stream will keep processing as long as the device stays
    // connected. Once they disconnect, then...
    on_device_disconnected(my_id, &devices).await;
}

///
async fn on_device_message(my_id: usize, msg: Message, context: &CallContext, devices: &Devices) {
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
    // let result = context.db.lsave_packet(&context, &packet).await;
    let result = context.db.save_packet("ws", &packet);
    if result.is_err() {
        println!("ws: [device_message] save_packet_string failed");
        return;
    }

    // 2) post action payload to ship. event source receiver will relay any updates/effects
    //     back to connected devices
    // is this the packet an action payload? if so, post to ship.
    let actions: serde_json::Result<Vec<ShipAction>> = serde_json::from_str(msg);
    if actions.is_ok() {
        let result = context.ship.lock().await.post(&packet).await;

        if result.is_err() {
            println!("ws: [device_message] proxy.post call failed. {:?}", result);
            return;
        }
    }

    // send the proxy post response back to the originating device over websocket
    let tx = devices.read();
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
    for (&uid, tx) in devices.read().await.iter() {
        if my_id == uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn on_ship_message(my_id: usize, msg: JsonValue, devices: &Devices) {
    // New message from the ship, send it to all connected devices (except same uid)...
    for (&uid, tx) in devices.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.as_str().unwrap())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn on_device_disconnected(my_id: usize, devices: &Devices) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    devices.write().await.remove(&my_id);
}

#[cfg(test)]
mod tests {
    use tungstenite::{client::IntoClientRequest, connect};
    // use url::Url;
    #[test]
    // connect to this node's websocket server (for receiving events from a ship)
    fn can_ws_connect() {
        println!("ws: [test][can_ws_connect] connecting to node websocket...");
        let mut request = "ws://127.0.0.1:3030/ws".into_client_request().unwrap();
        let headers = request.headers_mut();
        headers.insert("cookie", "urbauth-~ralbes-mislec-lodlev-migdev=0v4.tott3.pnoms.o4bb5.tjo4n.1ve3m; Path=/; Max-Age=604800".parse().unwrap());
        let (mut socket, _response) = connect(request).unwrap(); // .expect("Can't connect");
        loop {
            println!("ws: [test][can_ws_connect] waiting for ship events...");
            let msg = socket.read_message().expect("Error reading message");
            println!("Received: {}", msg);
        }
    }
}
