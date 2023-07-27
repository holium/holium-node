use futures_util::{SinkExt, StreamExt, TryFutureExt};
use lazy_static::lazy_static;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use tokio::task::JoinHandle;

use colored::*;
use colored_json::to_colored_json_auto;

use tokio::sync::RwLock;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::context::CallContext;

/// global unique device id counter.
static NEXT_DEVICE_ID: AtomicUsize = AtomicUsize::new(1);
/// global unique message id counter.
static NEXT_MESSAGE_ID: AtomicU64 = AtomicU64::new(1);

/// currently connected devices.
///
/// - key is the device id (based on NEXT_DEVICE_ID)
/// - value is a sender of `warp::ws::Message` which sends messages
///    across the underlying channel to the websocket device sender
type DeviceMap = HashMap<usize, crossbeam::channel::Sender<Message>>;
type Devices = Arc<RwLock<DeviceMap>>;

#[derive(Debug, Clone)]
struct MsgEntry {
    #[allow(dead_code)] // id "not read" warning. disable for now.
    // unique message id managed by the holon
    id: u64,
    // id of the message originating on the device (e.g. urbit action message id)
    source_id: u64,
    // id of connected device (key into the DeviceMap)
    device_id: usize,
}

// maps holong managed message ids to origin message ids and device
type MsgMap = HashMap<u64, MsgEntry>;

// thread-safe store of the message map
// type MsgStore = Arc<RwLock<MsgMap>>;

lazy_static! {
    //
    // singleton ship listener managed by the websocket endpoint
    // only one ship listener is needed per holon process. each ship event is mapped
    //   by to the originating device use the MsgMap as a lookup (holon_id -> urbit_urbit_message/device pair)
    //
    // addt'l: a ship listener thread will only exist if at least on device is connected over websocket
    //    once all devices are disconnected, the ship listener is terminated, but is respawned on subsequent
    //    websocket device connections
    //
    static ref SHIP_RECEIVER: Arc<RwLock<Option<JoinHandle::<()>>>> = Arc::new(RwLock::new(None));
    static ref MESSAGE_STORE: Arc<RwLock<MsgMap>> = Arc::new(RwLock::new(MsgMap::new()));

}

// @see: https://developers.urbit.org/reference/arvo/eyre/external-api-ref#responses
//
// handles ship responses which are delivered in a variety of flavors
#[derive(Debug, Deserialize, Serialize)]
struct ShipResponse {
    id: u64,
    response: String,
    json: Option<JsonValue>,
    err: Option<String>,
    ok: Option<String>,
}

// @see: https://developers.urbit.org/reference/arvo/eyre/external-api-ref#actions
//
// loose representation of a ship action which come in a variety of flavors (e.g. poke, subscribe, etc...)
//   where some of the fields are missing/present depending on the flavor.
#[derive(Debug, Deserialize, Serialize)]
pub struct ShipAction {
    id: u64,
    action: String,
    ship: String,
    app: String,
    mark: Option<String>,
    json: Option<JsonValue>,
    path: Option<String>,
}

// authorization tokens are required as part of the websocket handshake

// MissingAuthToken is the rejection that is raised when the authorization
// token is missing from the request
#[derive(Debug)]
struct MissingAuthToken;
impl warp::reject::Reject for MissingAuthToken {}

// InvalidAuthToken is the rejection that is raised when the authorization
// token is provided but deemed to be invalid (e.g. expired or improperly formatted, etc.)
#[derive(Debug)]
struct InvalidAuthToken;
impl warp::reject::Reject for InvalidAuthToken {}

pub async fn start(
    context: CallContext,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // "filterize" our state

    let devices = Devices::default();
    let devices = warp::any().map(move || devices.clone());

    let with_context = warp::any().map(move || context.clone());

    // GET /ws -> websocket upgrade
    let handler = warp::path!("ws")
        .and(warp::header::headers_cloned())
        .and(with_context)
        .and_then(
            /*
              ensure that there is a cookie header and that the cookie contains a key=value pair
                containing this ship's patp

               e.g. - ensure cooke exists where:
                   urbauth-<ship patp here>=<auth token here>

                 here is a sample cookie string that passes:
                   "urbauth-~ralbes-mislec-lodlev-migdev=0v6.bb0bl.hiu64.et7nk.qljtl.hdurg; Path=/; Max-Age=604800"
            */
            |headers: HeaderMap, context: CallContext| async move {
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
                Ok(context)
            },
        )
        .and(devices)
        .and(warp::ws())
        .map(|context: CallContext, devices: Devices, ws: warp::ws::Ws| {
            // This will call our function if the handshake succeeds.
            println!(
                "ws: [ws_upgrade] - [tx, rx]] addresses [{:p}, {:p}]",
                &context.sender, &context.receiver
            );
            ws.on_upgrade(move |socket| device_connected(socket, devices, context.clone()))
        });

    handler
}

async fn device_connected(
    ws: WebSocket,
    devices: Devices,
    context: CallContext, /*ship_event_receiver: ShipReceiver*/
) {
    // use a counter to assign a new unique ID for this device.
    let my_id = NEXT_DEVICE_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut device_ws_tx, mut device_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    // let (tx, mut rx) = mpsc::unbounded_channel();
    let (tx, rx) = crossbeam::channel::unbounded();
    // let mut rx = UnboundedReceiverStream::new(rx);

    // spawn a task to listen for messages to send to transmit to connected devices
    tokio::task::spawn(async move {
        println!("ws: [device_connected] waiting for outgoing messages...");
        while let Ok(message) = rx.recv() {
            println!("ws: [device_connected] sending message to device...");
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

    // one and only one ship listener per holon process
    if SHIP_RECEIVER.read().await.is_none() {
        println!("ws: [device_connected] starting ship listener...");

        let ship_rx_context = context.clone();
        let ship_rx_devices = devices.clone();

        // ingest messages coming from the ship's SSE
        let handle = tokio::task::spawn(async move {
            println!("ws: [device_connected] waiting for ship event...");

            while let Ok(result) = ship_rx_context.receiver.recv() {
                println!(
                    "ws: [device_connected] received event from ship => [{}, {}]",
                    my_id, result
                );
                on_ship_message(my_id, result, &ship_rx_devices).await;
            }
        });

        SHIP_RECEIVER.write().await.replace(handle);
    }

    // listen for message from connected devices
    println!("ws: [device_connected] waiting for device message...");
    while let Some(result) = device_ws_rx.next().await {
        println!("{:?}", result);
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        // process the incoming device message
        on_device_message(my_id, msg, &context, &devices).await;
    }
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

    #[cfg(feature = "trace")]
    println!("ws: [device_message] [{}, {}]", my_id, msg);

    let packet: serde_json::Result<JsonValue> = serde_json::from_str(msg);

    if packet.is_err() {
        println!("ws: [device_message] payload not valid json");
        return;
    }

    let packet = packet.unwrap();

    // 1) save packet payload to db
    let result = context.db.save_packet("ws", &packet);
    if result.is_err() {
        println!("ws: [device_message] save_packet_string failed");
        return;
    }

    // 2) Determine if standard urbit action message
    //
    //   Urbit actions are submitted as a Json array; therefore any message
    //   that comes in from a device is a json array is, for now, treated as an urbit action.
    //   If the message is anything other than an array, it's considered a holon message
    if packet.is_array() {
        // 2) post action payload to ship. event source receiver will relay any updates/effects
        //     back to connected devices

        // the flow should follow:
        //   device -[req]-> node -[req]-> ship -[resp]-> node -[resp]-> device

        // leverage serde_json deserialization strictly for the validation of the message
        //  if nothing else
        let actions: serde_json::Result<Vec<ShipAction>> = serde_json::from_str(msg);

        if actions.is_err() {
            println!(
                "ws: [device_message] error deserializing message to action array: {}",
                msg
            );
            return;
        }

        let actions = actions.unwrap();

        // to prevent orphaned messages (ship post that succeeds but MESSAGE_STORE persist fails),
        //   add the holon id <-> urbit id message map entry first. that way if the ship post
        //   below fails, it may orphan the mapping entry but the message post can still be retried on error
        let msg_id = NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
        MESSAGE_STORE.write().await.insert(
            actions[0].id,
            MsgEntry {
                id: msg_id,
                source_id: actions[0].id,
                device_id: my_id,
            },
        );

        #[cfg(feature = "trace")]
        println!(
            "{}: {} relaying actions payload:",
            "[ws]".bright_yellow(),
            "[device_message]".bright_blue()
        );
        #[cfg(feature = "trace")]
        println!("{}", to_colored_json_auto(&packet).unwrap());

        let result = context.ship.lock().await.post(&packet).await;

        if result.is_err() {
            // an error here is a big deal. print to holon std out...
            println!(
                "[ws]: [device_message] proxy.post call failed. {:?}",
                result
            );

            // ...and send error response to connected device over socket
            return;
        }

        // no more to do. eventually a response to ship requests will come back thru the
        //   SHIP_RECEIVER where it will be delivered to the device from whence it originated
        //   (assuming the device is still connected to the socket)
    } else {
        // holon message
        // this is json object message; therefore it's destination is meant for holon
        let value: JsonValue = serde_json::from_value(packet).unwrap();

        // for now, echo holon messages back to calling device (no use case for holon
        //  messages at the time of writing)
        //  holon message flow: device - [req] -> holon -> [res] -> device

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
            let _ = tx.send(Message::text(value.to_string()));
        }
    }

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

// msg_id - the holon managed message id
async fn find_msg_entry(msg_id: u64) -> Option<MsgEntry> {
    let lock = MESSAGE_STORE.read().await;
    let entry = lock.get(&msg_id);
    if entry.is_none() {
        return None;
    }
    let entry = entry.unwrap();
    Some(entry.clone())
}

async fn find_device_tx(
    device_id: usize,
    devices: &Devices,
) -> Option<crossbeam::channel::Sender<Message>> {
    let lock = devices.read().await;
    let tx = lock.get(&device_id);
    if tx.is_none() {
        return None;
    }
    Some(tx.unwrap().clone())
}

async fn on_ship_message(_my_id: usize, msg: JsonValue, devices: &Devices) {
    let data = serde_json::from_value::<ShipResponse>(msg.clone());
    if data.is_err() {
        println!(
            "ws: [on_ship_message] error deserializing ship event => {:?}",
            serde_json::to_string_pretty(&msg)
        );
        return;
    }

    let mut data = data.unwrap();
    // note the id coming from the ship will be the holon managed message
    // id. use it to find the corresponding MsgStore entry which provides
    // the originating message id (e.g. urbit message id)
    let entry = find_msg_entry(data.id).await;

    if entry.is_none() {
        println!("ws: [on_ship_message] message {} not found", data.id);
        return;
    }

    let entry = entry.unwrap();
    let tx = find_device_tx(entry.device_id, devices).await;

    if tx.is_none() {
        println!("ws: [on_ship_message] device {} not found", entry.device_id);
    }

    // override the outgoing message's id field with the original message id
    data.id = entry.source_id;

    let result = serde_json::to_string::<ShipResponse>(&data);

    if result.is_err() {
        println!("ws: [on_ship_message] error serializing message {:?}", data);
    }

    let tx = tx.unwrap();
    if let Err(_disconnected) = tx.send(Message::text(result.unwrap())) {
        // The tx is disconnected, our `user_disconnected` code
        // should be happening in another task, nothing more to
        // do here.
    }
}

async fn on_device_disconnected(my_id: usize, devices: &Devices) {
    eprintln!("ws: [on_device_disconnected] removing device {}...", my_id);

    // stream closed up, so remove from the device list
    devices.write().await.remove(&my_id);

    if devices.read().await.len() == 0 {
        #[cfg(feature = "trace")]
        println!("no more connected devices. stopping ship listener...");

        // kill the current ship receiver thread
        SHIP_RECEIVER.read().await.as_ref().unwrap().abort();

        #[cfg(feature = "trace")]
        println!("after no more connected devices. stopping ship listener...");

        let mut opt = SHIP_RECEIVER.write().await;
        *opt = None;
    }
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
                            println!("[thread-{}]: received actions - {:?}", i, actions[i].json);
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
