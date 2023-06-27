use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use warp_real_ip::get_forwarded_for;

use crate::dispatcher::Dispatcher;

pub fn ws_route(
    dispatcher: Dispatcher,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let signaling = warp::path!("hol" / "ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(get_forwarded_for())
        .and(warp::query::<HashMap<String, String>>())
        .and(with_dispatcher(dispatcher))
        .map(
            |ws: warp::ws::Ws,
             _remote_ip: Option<SocketAddr>,
             _forwarded_for_ips: Vec<IpAddr>,
             _query_params: HashMap<String, String>,
             dispatcher: Dispatcher| {
                ws.on_upgrade(move |socket| handle_sync(socket, dispatcher))
            },
        );
    signaling
}

pub async fn handle_sync(ws: WebSocket, dispatcher: Dispatcher) {
    println!("establishing holon node link");
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (sender, receiver) = mpsc::unbounded_channel();
    let mut receiver = UnboundedReceiverStream::new(receiver);

    tokio::task::spawn(async move {
        while let Some(message) = receiver.next().await {
            ws_sender
                .send(message)
                .unwrap_or_else(|e| {
                    println!("[signaling] websocket send error: {}", e);
                    // todo: handle cleanup
                })
                .await;
        }
    });

    while let Some(result) = ws_receiver.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                println!("[signaling] websocket error: {}", e);
                break;
            }
        };
        if let Ok(message) = message.to_str() {
            handle_message(sender.clone(), dispatcher.clone(), message).await;
        };
    }
}

pub async fn handle_message(
    sender: UnboundedSender<Message>,
    mut dispatcher: Dispatcher,
    message: &str,
) {
    let message: Value = serde_json::from_str(message).expect("Error parsing message");
    let metadata = message["meta"].as_object().unwrap();
    let path = metadata["path"].as_str().unwrap();
    let contents = message["contents"].clone(); //.as_object().unwrap();
                                                // use the dispatcher to route/handle the incoming message
    let result = dispatcher
        .dispatch(sender, path.to_string(), contents)
        .await;
    if result.is_err() {
        println!("error dispatching message: {:?}", result);
    }
}

fn with_dispatcher(
    dispatcher: Dispatcher,
) -> impl Filter<Extract = (Dispatcher,), Error = Infallible> + Clone {
    warp::any().map(move || dispatcher.clone())
}
