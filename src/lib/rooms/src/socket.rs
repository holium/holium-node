use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use warp_real_ip::get_forwarded_for;

use crate::types::{Peer, PeerId, PeerIp, Response, Room, RoomTuple, ROOM_MAP};

pub fn signaling_route(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let signaling = warp::path("signaling")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(get_forwarded_for())
        .and(warp::query::<HashMap<String, String>>())
        .map(
            |ws: warp::ws::Ws,
             remote_ip: Option<SocketAddr>,
             peer_ips: Vec<IpAddr>,
             query_params: HashMap<String, String>| {
                let peer_ip = match peer_ips.first() {
                    Some(ip) => ip.to_string(),
                    None => remote_ip.unwrap().ip().to_string(),
                };

                // Get the query parameter
                let peer_id = query_params
                    .get("serverId")
                    .unwrap_or(&String::from("default_value"))
                    .clone();

                ws.on_upgrade(move |socket| handle_signaling(socket, peer_ip, peer_id))
            },
        );
    signaling
}

pub async fn handle_signaling(ws: WebSocket, peer_ip: PeerIp, peer_id: String) {
    println!(
        "[signaling] peer {} connected from {}",
        peer_id.clone(),
        peer_ip
    );
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (sender, receiver) = mpsc::unbounded_channel();
    let mut receiver = UnboundedReceiverStream::new(receiver);

    // let cloned_id = peer_id.clone();

    tokio::task::spawn(async move {
        while let Some(message) = receiver.next().await {
            ws_sender
                .send(message)
                .unwrap_or_else(|e| {
                    println!("[signaling] websocket send error: {}", e);
                    // peer_leave(&cloned_id)
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
            handle_message(sender.clone(), &peer_ip, &peer_id, message).await;
        };
    }

    // peer_leave(&peer_id);
    // peer_broadcast(&peer_ip);
}

pub async fn handle_message(
    sender: UnboundedSender<Message>,
    peer_ip: &PeerIp,
    peer_id: &PeerId,
    message: &str,
) {
    let message: Value = serde_json::from_str(message).expect("Error parsing message");
    match message["type"].as_str().unwrap() {
        // Receive peer info from the client
        // "sign" => {
        //     peer_sign(peer_id, peer_ip, sender, message);
        //     peer_broadcast(peer_ip);
        // }
        // "find" => peer_find(sender, message),
        // "call" => peer_call(peer_id, sender, message),
        "create-room" => {
            println!("{} - create-room - {}", peer_id, peer_ip);
            let mut peer_map = HashMap::new();
            peer_map.insert(
                peer_id.clone().to_string(),
                (
                    peer_ip.clone().to_string(),
                    sender.clone(),
                    Peer {
                        id: peer_id.clone(),
                    },
                ),
            );

            let present: Vec<String> = peer_map.keys().cloned().collect();
            let new_room = Room {
                rid: format!("room-{}", peer_id).to_string(),
                title: peer_id.clone(),
                creator: peer_id.clone(),
                provider: "default".to_string(),
                access: "public".to_string(),
                present: present,
                whitelist: Vec::new(),
                capacity: 10,
                path: None,
            };
            let rid = new_room.rid.clone();
            let peer_map = Arc::new(RwLock::new(peer_map));
            let new_room = Arc::new(RwLock::new(new_room));
            let mut map = ROOM_MAP.write().unwrap();
            map.insert(rid, RoomTuple::from((peer_map, new_room)));
        }
        "edit-room" => {
            println!("{} - create-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let mut rooms = ROOM_MAP.write().unwrap();
            let (room_peers, room) = match rooms.get_mut(&rid) {
                Some(room) => room,
                None => return,
            };
            let mut room = room.write().unwrap();
            match message["title"].as_str() {
                Some(title) => room.title = title.to_string(),
                None => (),
            };
            match message["access"].as_str() {
                Some(access) => room.access = access.to_string(),
                None => (),
            };
            match message["capacity"].as_u64() {
                Some(capacity) => room.capacity = capacity.clone() as u32,
                // cast to u32
                None => (),
            };
            if room.creator != peer_id.clone() {
                return;
            }

            let peer_map = room_peers.write().unwrap();
            for (_, sender, _) in peer_map.values() {
                let message = json!({
                    "type": "edit-room",
                    "title": room.title.clone(),
                    "access": room.access.clone(),
                    "capacity": room.capacity.clone(),
                });
                sender.send(Message::text(message.to_string())).unwrap();
            }
        }
        "delete-room" => {
            println!("{} - delete-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let mut rooms = ROOM_MAP.write().unwrap();
            let (room_peers, _) = match rooms.remove(&rid) {
                Some(room) => room,
                None => return,
            };
            let mut peer_map = room_peers.write().unwrap();
            for (_, (_, sender, _)) in peer_map.drain() {
                let message = json!({
                    "type": "delete-room",
                    "rid": rid.clone(),
                });
                sender.send(Message::text(message.to_string())).unwrap();
            }
        }
        "enter-room" => {
            println!("{} - enter-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let rooms = ROOM_MAP.read().unwrap();
            let (room_peers, room) = match rooms.get(&rid) {
                Some(room) => room,
                None => return,
            };
            let mut peer_map = room_peers.write().unwrap();
            if peer_map.contains_key(peer_id) {
                return;
            }
            peer_map.insert(
                peer_id.clone().to_string(),
                (
                    peer_ip.clone().to_string(),
                    sender.clone(),
                    Peer {
                        id: peer_id.clone(),
                    },
                ),
            );
            let present: Vec<String> = peer_map.keys().cloned().collect();
            let mut room = room.write().unwrap();
            room.present = present;
            // send the message to all peers in the room
            let message = json!({
                "type": "room-entered",
                "rid": rid.clone(),
                "peer_id": peer_id.clone(),
                "room": &room.clone(),
            });
            for (_, sender, _) in peer_map.values() {
                sender.send(Message::text(message.to_string())).unwrap();
            }
        }
        "leave-room" => {
            println!("{} - leave-room - {}", peer_id, peer_ip);
            // TODO add check if the peer is signed
            let rid = message["rid"].as_str().unwrap().to_string();
            let rooms = ROOM_MAP.read().unwrap();
            let (room_peers, room) = match rooms.get(&rid) {
                Some(room) => room,
                None => return,
            };
            let mut peer_map = room_peers.write().unwrap();
            if !peer_map.contains_key(peer_id) {
                return;
            }
            peer_map.remove(peer_id);
            let present: Vec<String> = peer_map.keys().cloned().collect();
            let mut room = room.write().unwrap();
            room.present = present;
            let message = serde_json::to_string(&room.clone()).unwrap();
            let message = Message::text(message);
            for (_, sender, _) in peer_map.values() {
                sender.send(message.clone()).unwrap();
            }
        }
        "signal" => {
            println!("{} - signal - {}", peer_id, peer_ip);

            // match message["signal_type"] {
            //     "answer" | "error" | "sdp-offer" | "sdp-answer" | "ice-candidate" | "media"
            //     | "exchange-a" | "exchange-b" => relay(message),
            // }
        }
        // TODO you need to add peers if they are not in a
        "connect" => {
            println!("{} - connect - {}", peer_id, peer_ip);
            // 1. get all rooms from map
            let rooms = ROOM_MAP.read().unwrap();
            // 2. form rooms to json
            let rooms: Vec<Room> = rooms
                .iter()
                .map(|(_, (_, room))| room.write().unwrap().clone())
                .collect();
            // 2. send rooms to new peer
            let message = json!({
                "type": "rooms",
                "rooms": rooms,
            });
            sender.send(Message::text(message.to_string())).unwrap();
        }
        // "disconnect" => {
        //     ALL_PEERS.write().unwrap().remove(peer_id);
        // }
        "disconnect" => {
            println!("{} - disconnect - {}", peer_id, peer_ip);
            let mut rooms = ROOM_MAP.write().unwrap();
            for (_, (room_peers, _)) in rooms.iter_mut() {
                let mut peer_map = room_peers.write().unwrap();
                peer_map.remove(peer_id);
            }
        }
        // "connect" | "answer" | "error" | "sdp-offer" | "sdp-answer" | "ice-candidate" | "media"
        // | "exchange-a" | "exchange-b" => relay(message),
        "ping" => pong(sender),
        _ => unknown(),
    };
}

fn relay(message: Value) {
    let id = message["id"].as_str().unwrap().to_string();
    let rtype = message["type"].as_str().unwrap().to_string();
    let rid = message["rid"].as_str().unwrap().to_string();
    let rooms = ROOM_MAP.read().unwrap();
    let room = match rooms.get(&rid) {
        Some(room) => room,
        None => return,
    };
    let (peers, _) = room;

    let map = peers.read().unwrap();
    match map.get(&id) {
        Some((_, sender, _)) => {
            println!("[signaling] relay {} message to {}", rtype, id);
            sender
                .send(Message::text(json!(message).to_string()))
                .unwrap();
        }
        None => (),
    };
}

fn pong(sender: UnboundedSender<Message>) {
    sender
        .send(Message::text(json!({ "type": "pong" }).to_string()))
        .expect("Failed to send pong")
}

fn unknown() {
    println!("[signaling] unknown message type")
}
