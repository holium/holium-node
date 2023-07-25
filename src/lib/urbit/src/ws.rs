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

use serde_json::json;
use tokio::sync::mpsc::UnboundedReceiver;

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

    println!(
        "ws: [start] - [tx, rx]] addresses [{:p}, {:p}]",
        &context.sender, &context.receiver
    );
    let with_context = warp::any().map(move || context.clone());

    // GET /chat -> websocket upgrade
    let handler = warp::path!("ws")
        .and(warp::header::headers_cloned())
        .and(with_context)
        .and(devices)
        .and_then(
            /*
              ensure that there is a cookie header and that the cookie contains a key=value pair
                containing this ship's patp

               e.g. - ensure cooke exists where:
                   urbauth-<ship patp here>=<auth token here>

                 here is a sample cookie string that passes:
                   "urbauth-~ralbes-mislec-lodlev-migdev=0v6.bb0bl.hiu64.et7nk.qljtl.hdurg; Path=/; Max-Age=604800"
            */
            |headers: HeaderMap, context: CallContext, devices: Devices| async move {
                if !headers.contains_key("cookie") {
                    return Err(warp::reject::custom(MissingAuthToken));
                }
                let cookie_key = format!(
                    "urbauth-~{}",
                    context.ship.lock().await.ship_name.as_ref().unwrap()
                );
                #[cfg(feature = "trace")]
                println!("ws: [start] searching cookie for token '{}'...", cookie_key);
                let cookie_str = headers.get("cookie").unwrap().to_str().unwrap();
                let parts = cookie_str.split(";");
                let mut auth_token: Option<&str> = None;
                for part in parts {
                    let pair: Vec<&str> = part.split("=").collect();
                    if pair[0] == cookie_key {
                        auth_token.replace(pair[1]);
                        break;
                    }
                }
                if auth_token.is_none() {
                    return Err(warp::reject::custom(MissingAuthToken));
                }
                #[cfg(feature = "trace")]
                println!("ws: [start] token => {}", auth_token.unwrap());
                Ok((context, devices))
            },
        )
        .and(warp::ws())
        .map(
            |(context, devices): (CallContext, Devices), ws: warp::ws::Ws| {
                // This will call our function if the handshake succeeds.
                println!(
                    "ws: [ws_upgrade] - [tx, rx]] addresses [{:p}, {:p}]",
                    &context.sender, &context.receiver
                );
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
        println!("ws: [device_connected] waiting for outgoing messages...");
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
        println!("ws: [device_connected] waiting for device message...");
        // Every time the device sends a message, broadcast it to
        // all other devices...
        while let Some(result) = device_ws_rx.next().await {
            println!("{:?}", result);
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

    // let ship_receiver = context.ship.lock().await.open_channel().await;

    // if ship_receiver.is_err() {
    //     println!("ws: [device_connected] open_channel call failed");
    // }

    // let ship_receiver = ship_receiver.unwrap();
    // let dev = devices.clone();

    // let _ = tokio::spawn(async move {
    //     loop {
    //         println!("ship: [listen] waiting for ship event...",);

    //         let msg = ship_receiver.recv();

    //         if msg.is_err() {
    //             println!("ship: [listen] event receive error. msg => {:?}", msg.err());
    //             continue;
    //         }

    //         let msg = msg.unwrap();

    //         if msg.is_err() {
    //             println!("ship: [listen] event request error. msg => {:?}", msg);

    //             let msg = json!({
    //               "id": 4884,
    //               "type": "error",
    //               "error": "ship-stream-disconnected",
    //             });

    //             println!("ship: [listen] forwarding error to devices => {}", msg);

    //             on_ship_message(my_id, msg, &dev).await;

    //             continue;
    //         }

    //         // the deserialized Event from SSE
    //         let event = msg.unwrap();

    //         #[cfg(feature = "trace")]
    //         println!("ship: [listen] received event => {}", event);

    //         let data = serde_json::from_str(&event.data);

    //         if data.is_err() {
    //             println!("ship: [listen] error deserializing event source message to json");
    //             continue;
    //         }

    //         let data = data.unwrap();

    //         // log the entire packet to the database
    //         let _ = context.db.save_packet("ship", &data);

    //         #[cfg(feature = "trace")]
    //         println!("ship: [listen] sending event to receiver => {}", data);

    //         // let send_result = tx.send(Message::text(data.to_string()));
    //         on_ship_message(my_id, data, &dev).await;
    //     }
    // })
    // .await;

    /////////////////////
    //
    // SHIP update/event listener
    //  mpsc (multi-producer single consumer) means that there should only ever be one
    //   ship listening receiver; therefore safe to lock (i.e. no race conditions or other blocking concerns
    // let ship_receiver = Arc::clone(&context.receiver);
    println!("ws: [device_connected] waiting for ship event...");
    // let mut ship_receiver = context.receiver.lock().await;
    // loop {
    //     let result = ship_receiver.recv().await;
    //     if result.is_none() {
    //         break;
    //     }
    //     let result = result.unwrap();
    //     println!(
    //         "ws: [device_connected] received event from ship => [{}, {}]",
    //         my_id, result
    //     );
    //     on_ship_message(my_id, result, &devices).await;
    // }
    let rx_devices = devices.clone();
    let _ = tokio::task::spawn(async move {
        let mut ws_receiver = context.receiver.lock().await;
        println!("GOT RECIEVER");
        {
            while let Some(result) = ws_receiver.recv().await {
                println!(
                    "ws: [device_connected] received event from ship => [{}, {}]",
                    my_id, result
                );
                on_ship_message(my_id, result, &rx_devices).await;
            }
        }
    })
    .await;

    //
    /////////////////////

    // device_ws_rx stream will keep processing as long as the device stays
    // connected. Once they disconnect, then...
    on_device_disconnected(my_id, &devices).await;
}

///
async fn on_device_message(my_id: usize, msg: Message, context: &CallContext, _devices: &Devices) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    #[cfg(feature = "trace")]
    println!("ws: [device_message] [{}, {}]", my_id, msg);

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
    // let actions: serde_json::Result<Vec<ShipAction>> = serde_json::from_str(msg);

    // if actions.is_err() {
    //     println!(
    //         "ws: [device_message] error deserializing message to action array: {}",
    //         msg
    //     );
    //     return;
    // }

    println!("ws: [device_message] relaying actions payload to ship...");

    let result = context.ship.lock().await.post(&packet).await;

    if result.is_err() {
        println!("ws: [device_message] proxy.post call failed. {:?}", result);
        return;
    }

    // disable sending messages back to the calling device
    // the flow should follow:
    //   device -[req]-> node -[req]-> ship -[resp]-> node -[resp]-> device
    // send the proxy post response back to the originating device over websocket
    // let tx = devices.read();
    // let tx = tx.await;
    // let tx = tx.get(&my_id);
    // {
    //     if tx.is_none() {
    //         println!("ws: [device_message] error attempting to read device {} from list of connected devices", my_id);
    //         return;
    //     }
    //     let tx = tx.unwrap();
    //     let _ = tx.send(Message::text(msg.clone()));
    // }

    ////////////////////////////////////////////////////////
    // disable broadcast for now
    // New message from this user, send it to everyone else (except same uid)...
    // for (&uid, tx) in devices.read().await.iter() {
    //     if my_id == uid {
    //         if let Err(_disconnected) = tx.send(Message::text(msg.clone())) {
    //             // The tx is disconnected, our `user_disconnected` code
    //             // should be happening in another task, nothing more to
    //             // do here.
    //         }
    //     }
    // }
    ///////////////////////////////////////////////////////
}

async fn on_ship_message(my_id: usize, msg: JsonValue, devices: &Devices) {
    // New message from the ship, send it to all connected devices (except same uid)...
    for (&uid, tx) in devices.read().await.iter() {
        if my_id == uid {
            if let Err(_disconnected) = tx.send(Message::text(msg.to_string())) {
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
    use futures_util::{SinkExt, StreamExt};
    use rand::Rng;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
    use tokio::time::{sleep, Duration};
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{client::IntoClientRequest, connect, Message},
    };
    static NEXT_MSG_ID: AtomicUsize = AtomicUsize::new(0);
    static SENT_COUNT: AtomicUsize = AtomicUsize::new(0);
    static RECD_COUNT: AtomicUsize = AtomicUsize::new(0);
    #[test]
    // connect to this node's websocket server (for receiving events from a ship)
    fn can_ws_connect() {
        println!("ws: [test][can_ws_connect] connecting to node websocket...");
        let mut request = "ws://127.0.0.1:3030/ws".into_client_request().unwrap();
        let headers = request.headers_mut();
        headers.insert("cookie", "urbauth-~ralbes-mislec-lodlev-migdev=0v6.s58oo.vp1c4.e4fg8.peu65.mols9; Path=/; Max-Age=604800".parse().unwrap());
        let (mut socket, _response) = connect(request).unwrap();
        loop {
            println!("ws: [test][can_ws_connect] waiting for ship events...");
            let msg = socket.read_message().expect("Error reading message");
            println!("received: {}", msg);
        }
    }

    ///
    /// test_ws_multi_connect - open NUM_WS_CONNECTIONS websocket client connections
    ///   and succeed only if 8 unique sends are transmitted and 8 unique receives are
    ///   read.
    ///
    ///  prereqs: you must start the main process which will start the websocket server and
    ///    serve out the ws routes.
    ///
    #[tokio::test]
    async fn test_ws_multi_connect() {
        const NUM_WS_CONNECTIONS: usize = 8;
        let (tx, mut rx): (UnboundedSender<String>, UnboundedReceiver<String>) =
            unbounded_channel();

        println!("starting writer...");
        // tokio::task::spawn(async move {
        for i in 0..NUM_WS_CONNECTIONS {
            let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_entropy();

            let mut request = "ws://127.0.0.1:3030/ws".into_client_request().unwrap();
            let headers = request.headers_mut();
            headers.insert("cookie", "urbauth-~ralbes-mislec-lodlev-migdev=0v6.s58oo.vp1c4.e4fg8.peu65.mols9; Path=/; Max-Age=604800".parse().unwrap());

            let (socket, _) = connect_async(request).await.unwrap();

            let (mut itx, mut irx) = socket.split();
            let tx1 = tx.clone();
            let tx2 = tx.clone();

            tokio::task::spawn(async move {
                println!("[thread-{}] - write started", i);
                // loop {
                let secs = rng.gen_range(0..10);
                sleep(Duration::from_secs(secs)).await;
                let msg_id = NEXT_MSG_ID.fetch_add(1, Ordering::Relaxed);
                let msg = json!([{
                  "id": msg_id,
                  "ship":"ralbes-mislec-lodlev-migdev",
                  "action":"poke",
                  "app": "helm",
                  "mark": "helm-hi",
                  "json": format!("test message {}", msg_id)
                }]);
                println!("[thread-{}]: sending message...", i); // [thread-{}]: {}", i, msg.to_string());
                let _ = itx.send(Message::text(msg.to_string())).await;
                SENT_COUNT.fetch_add(1, Ordering::Relaxed);
                let _ = tx1.send("".to_string());
                // }
            });
            tokio::task::spawn(async move {
                println!("[thread-{}] - read started", i);
                loop {
                    let msg = irx.next().await;
                    if msg.is_none() {
                        println!("[thread-{}]: received message [no data]", i);
                    } else {
                        let msg = msg.unwrap();

                        if msg.is_err() {
                            println!("[thread-{}]: received message [error]", i);
                        }

                        let msg = msg.unwrap();

                        println!("[thread-{}]: received message {}", i, msg);
                        let msg = msg.to_string();

                        let res = serde_json::from_str::<Vec<super::ShipAction>>(&msg);

                        if res.is_err() {
                            println!(
                                "[thread-{}]: received message [error deserializing message]",
                                i
                            );
                        }

                        let actions = res.unwrap();

                        for i in 0..actions.len() {
                            println!("[thread-{}]: received actions - {}", i, actions[i].json);
                        }

                        RECD_COUNT.fetch_add(1, Ordering::Relaxed);
                        let _ = tx2.send("".to_string());
                    }
                }
            });
        }

        println!("waiting for eof...");
        loop {
            let msg = rx.recv().await;
            let sent_count = SENT_COUNT.fetch_add(0, Ordering::Relaxed);
            let recd_count = RECD_COUNT.fetch_add(0, Ordering::Relaxed);
            println!("waiting for eof [{}, {}]...", sent_count, recd_count);
            if sent_count == NUM_WS_CONNECTIONS && recd_count == NUM_WS_CONNECTIONS {
                println!("eot {:?}", msg);
                break;
            }
        }
    }
}
