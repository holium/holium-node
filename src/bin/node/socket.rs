use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::ws::WebSocket;
use warp::Filter;

pub struct SocketData {
    pub timestamp: Instant,
    pub message: Value,
}

pub fn socket_route(
    socket_map: Arc<RwLock<HashMap<String, Mutex<WebSocket>>>>,
    queued_signals: Arc<RwLock<HashMap<String, Vec<SocketData>>>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("socket-io")
        .and(warp::ws())
        .and(warp::any().map(move || Arc::clone(&socket_map)))
        .and(warp::any().map(move || Arc::clone(&queued_signals)))
        .map(
            |ws: warp::ws::Ws,
             socket_map: Arc<RwLock<HashMap<String, Mutex<WebSocket>>>>,
             queued_signals: Arc<RwLock<HashMap<String, Vec<SocketData>>>>| {
                ws.on_upgrade(move |socket| async move {
                    let id = Uuid::new_v4().to_string();
                    socket_map
                        .write()
                        .await
                        .insert(id.clone(), Mutex::new(socket));
                    // TODO: implement logic similar to Node.js code.
                })
            },
        )
}
