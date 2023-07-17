// #![deny(warnings)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde_json::Value as JsonValue;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

/// Our global unique device id counter.
static NEXT_DEVICE_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected devices.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type DeviceMap = HashMap<usize, mpsc::UnboundedSender<Message>>;
type Devices = Arc<RwLock<DeviceMap>>;
type ShipReceiver = Arc<Mutex<UnboundedReceiver<JsonValue>>>;

lazy_static! {
    static ref CONNECTED_DEVICES: Devices = Arc::new(RwLock::new(HashMap::new()));
}

// static CONNECTED_DEVICES: Devices = Devices::new(RwLock::new(DeviceMap::new()));
// static CONNECTED_DEVICES: Lazy<DeviceMap> = Lazy::new(|| {
//     println!("initializing");
//     let mut m = HashMap::new();
//     m
// });

pub fn start(
    receiver: UnboundedReceiver<JsonValue>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Turn our "state" into a new Filter...
    // let users = warp::any().map(move || users.clone());

    // let devices = Devices::default();
    // let devices = warp::any().map(move || devices.clone());

    let receiver = Arc::new(Mutex::new(receiver));
    let ship_event_receiver = warp::any().map(move || receiver.clone());

    // GET /chat -> websocket upgrade
    let handler = warp::path!("ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        // .and(devices)
        .and(ship_event_receiver)
        .map(
            |ws: warp::ws::Ws, /*devices, */ ship_event_receiver: ShipReceiver| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| device_connected(socket, ship_event_receiver))
            },
        );

    handler
}

async fn device_connected(ws: WebSocket, ship_event_receiver: ShipReceiver) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_DEVICE_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut device_ws_tx, mut device_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

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

    // Save the sender in our list of connected users.
    CONNECTED_DEVICES.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

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
            device_message(my_id, msg).await;
        }
    });

    println!("ws: [device_connected] waiting for ship event...");
    // let ship_event_receiver = UnboundedReceiverStream::new(ship_receiver.);
    while let Some(result) = ship_event_receiver.lock().await.recv().await {
        ship_message(my_id, result).await;
    }

    // device_ws_rx stream will keep processing as long as the device stays
    // connected. Once they disconnect, then...
    device_disconnected(my_id).await;
}

async fn device_message(my_id: usize, msg: Message) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    println!("ws: [device_message] [{}, {}]", my_id, msg);

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

async fn ship_message(my_id: usize, msg: JsonValue) {
    println!(
        "ws: [device_connected] received event from ship => [{}, {}]",
        my_id, msg
    );
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

async fn device_disconnected(my_id: usize) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    CONNECTED_DEVICES.write().await.remove(&my_id);
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
