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

use crate::types::{Peer, PeerId, PeerIp, Room, PEER_MAP, ROOM_MAP};

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
            let rid = message["rid"].as_str().unwrap().to_string();
            let title = message["title"].as_str().unwrap().to_string();

            // path is optional
            let path = match message["path"].as_str() {
                Some(path) => Some(path.to_string()),
                None => None,
            };

            let new_room = Room {
                rid: rid,
                title: title,
                creator: peer_id.clone(),
                provider: "default".to_string(),
                access: "public".to_string(),
                present: vec![peer_id.clone()],
                whitelist: Vec::new(),
                capacity: 10,
                path: path,
            };
            let rid = new_room.rid.clone();

            // Add room to ROOM_MAP
            let new_room_lock = Arc::new(RwLock::new(new_room));
            {
                let mut rooms = ROOM_MAP.write().unwrap();
                rooms.insert(rid.clone(), new_room_lock);
            }

            // Prepare message
            let rooms_message = {
                let rooms = ROOM_MAP.read().unwrap();
                rooms
                    .iter()
                    .map(|(_, room)| {
                        let room = room.read().unwrap();
                        room.clone()
                    })
                    .collect::<Vec<Room>>()
            };

            let message = json!({
                "type": "rooms",
                "rooms": rooms_message,
            });

            // send update to all known peers
            let peers = PEER_MAP.read().unwrap();
            for (_, (_, sender, _)) in peers.iter() {
                sender.send(Message::text(message.to_string())).unwrap()
            }
        }

        "edit-room" => {
            println!("{} - edit-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => return,
            };
            let mut room = room.write().unwrap();
            if room.creator != peer_id.clone() {
                return;
            }
            match message["title"].as_str() {
                Some(title) => room.title = title.to_string(),
                None => (),
            };
            match message["access"].as_str() {
                Some(access) => room.access = access.to_string(),
                None => (),
            };
            match message["capacity"].as_u64() {
                Some(capacity) => room.capacity = capacity as u32,
                None => (),
            };

            let message = json!({
                "type": "edit-room",
                "title": room.title.clone(),
                "access": room.access.clone(),
                "capacity": room.capacity,
            });

            // send update to all known peers
            let peers = PEER_MAP.read().unwrap();
            for (_, (_, sender, _)) in peers.iter() {
                sender.send(Message::text(message.to_string())).unwrap()
            }
        }
        "delete-room" => {
            println!("{} - delete-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let mut rooms = ROOM_MAP.write().unwrap();
            match rooms.remove(&rid) {
                Some(room) => room,
                None => return,
            };

            // send update to all known peers
            let peers = PEER_MAP.read().unwrap();
            let message = json!({
                "type": "room-deleted",
                "rid": rid.clone(),
            });
            for (_, (_, sender, _)) in peers.iter() {
                sender.send(Message::text(message.to_string())).unwrap()
            }
        }
        "enter-room" => {
            println!("{} - enter-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();

            // Retrieve the room
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => return,
            };

            let mut room = room.write().unwrap();
            if room.present.contains(peer_id) {
                println!("{} already in room", peer_id);
                return;
            }

            room.present.push(peer_id.clone());

            // Create the message
            let message = json!({
                "type": "room-entered",
                "rid": rid.clone(),
                "peer_id": peer_id.clone(),
                "room": &room.clone(),
            });

            // send update to all known peers
            let peers = PEER_MAP.read().unwrap();
            for (_, (_, sender, _)) in peers.iter() {
                sender.send(Message::text(message.to_string())).unwrap()
            }
        }
        "leave-room" => {
            println!("{} - leave-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => return,
            };
            let mut room = room.write().unwrap();
            if let Some(index) = room.present.iter().position(|id| id == peer_id) {
                room.present.remove(index);
            }
            let message = serde_json::to_string(&room.clone()).unwrap();
            let message = Message::text(message);
            let peers = PEER_MAP.read().unwrap();
            for peer_id in room.present.iter() {
                if let Some((_, sender, _)) = peers.get(peer_id) {
                    sender.send(message.clone()).unwrap();
                }
            }
        }
        "signal" => {
            // println!("{:?}", message);
            // parse signal
            // example: Object {"from": String("~bus"), "msgId": Number(1685286533063), "signal": Object {"sdp": String("v=0\r\no=- 2917890842146306227 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=extmap-allow-mixed\r\na=msid-semantic: WMS\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 0.0.0.0\r\na=ice-ufrag:tCS5\r\na=ice-pwd:DiG00O4QI5+QFB1MjvAg9lTo\r\na=ice-options:trickle\r\na=fingerprint:sha-256 D6:20:CC:36:C0:85:0B:45:97:1F:6E:B9:80:62:87:2F:E6:2C:73:DD:40:4E:73:6D:63:A0:7D:D1:A6:91:94:16\r\na=setup:actpass\r\na=mid:0\r\na=sctp-port:5000\r\na=max-message-size:262144\r\n"), "type": String("offer")}, "to": String("~zod"), "type": String("signal")}
            let from = message["from"].as_str().unwrap().to_string();
            let to = message["to"].as_str().unwrap().to_string();
            let rid = message["rid"].as_str().unwrap().to_string();
            // check if they are in the same room first
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => {
                    println!("room not found {}", rid);
                    return;
                }
            };
            let room = room.read().unwrap();
            if !room.present.contains(&from) || !room.present.contains(&to) {
                println!("both peers not in room {}", rid);
                return;
            }
            let signal = message["signal"].clone();
            let peers = PEER_MAP.read().unwrap();
            if let Some((_, sender, _)) = peers.get(&to) {
                let message = json!({
                    "type": "signal",
                    "from": from,
                    "signal": signal,
                });
                sender.send(Message::text(message.to_string())).unwrap();
            }
        }
        "connect" => {
            println!("{} - connect - {}", peer_id, peer_ip);
            let rooms = ROOM_MAP.read().unwrap();
            let rooms: Vec<Room> = rooms
                .iter()
                .map(|(_, room)| room.read().unwrap().clone())
                .collect();
            let message = json!({
                "type": "rooms",
                "rooms": rooms,
            });
            sender.send(Message::text(message.to_string())).unwrap();

            let mut peers = PEER_MAP.write().unwrap();
            peers.insert(
                peer_id.clone().to_string(),
                (
                    peer_ip.clone().to_string(),
                    sender.clone(),
                    Peer {
                        id: peer_id.clone(),
                    },
                ),
            );
        }
        "disconnect" => {
            println!("{} - disconnect - {}", peer_id, peer_ip);
            let mut room_ids_to_remove = Vec::new();

            {
                let rooms = ROOM_MAP.read().unwrap();
                for (rid, room) in rooms.iter() {
                    let mut room = room.write().unwrap();
                    if let Some(index) = room.present.iter().position(|id| id == peer_id) {
                        room.present.remove(index);
                    }
                    // if the peer was the last one in the room or the owner of the room, mark the room for removal
                    if room.present.is_empty() || &room.creator == peer_id {
                        room_ids_to_remove.push(rid.clone());
                    }
                }
            }

            // Remove rooms in a separate pass to avoid the mutable borrow issue
            {
                let mut rooms = ROOM_MAP.write().unwrap();
                for rid in room_ids_to_remove {
                    rooms.remove(&rid);
                    // send update to all known peers
                    let peers = PEER_MAP.read().unwrap();
                    let message = json!({
                        "type": "room-deleted",
                        "rid": rid.clone(),
                    });
                    for (_, (_, sender, _)) in peers.iter() {
                        sender.send(Message::text(message.to_string())).unwrap()
                    }
                }
            }

            let mut peers = PEER_MAP.write().unwrap();
            peers.remove(peer_id);
            // print current peer ids
            println!("Current peers: {:?}", peers.keys());
        }
        // "connect" | "answer" | "error" | "sdp-offer" | "sdp-answer" | "ice-candidate" | "media"
        // | "exchange-a" | "exchange-b" => relay(message),
        "ping" => pong(sender),
        _ => unknown(),
    };
}

// fn relay(message: Value) {
//     let id = message["id"].as_str().unwrap().to_string();
//     let rtype = message["type"].as_str().unwrap().to_string();
//     let rid = message["rid"].as_str().unwrap().to_string();
//     let rooms = ROOM_MAP.read().unwrap();
//     let room = match rooms.get(&rid) {
//         Some(room) => room,
//         None => return,
//     };

//     let map = room.read().unwrap();
//     // make sure the peer is in the room
//     if !map.present.contains(&id) {
//         return;
//     }

//     match PEER_MAP.read().unwrap().get(&id) {
//         Some((_, sender, _)) => {
//             println!("[signaling] relay {} message to {}", rtype, id);
//             sender
//                 .send(Message::text(json!(message).to_string()))
//                 .unwrap();
//         }
//         None => (),
//     };
// }

fn pong(sender: UnboundedSender<Message>) {
    sender
        .send(Message::text(json!({ "type": "pong" }).to_string()))
        .expect("Failed to send pong")
}

fn unknown() {
    println!("[signaling] unknown message type")
}
