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

use crate::types::{Peer, PeerId, PeerIp, Room, RoomType, PEER_MAP, ROOM_MAP};

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

    let cloned_id = peer_id.clone();

    tokio::task::spawn(async move {
        while let Some(message) = receiver.next().await {
            ws_sender
                .send(message)
                .unwrap_or_else(|e| {
                    println!("[signaling] websocket send error: {}", e);
                    disconnect(cloned_id.as_str())
                    // TODO HANDLE ANY OTHER CLEANUP
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
        "create-room" => {
            println!("{} - create-room - {}", peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let rtype = message["rtype"].as_str().unwrap().to_string();
            let title = message["title"].as_str().unwrap().to_string();

            // path is optional
            let path = match message["path"].as_str() {
                Some(path) => Some(path.to_string()),
                None => None,
            };

            let new_room = Room {
                rid: rid,
                rtype: rtype,
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

            // send self a room-created message
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => return,
            };
            let room = room.read().unwrap();
            let message = json!({
                "type": "room-created",
                "room": room.clone(),
            });
            sender.send(Message::text(message.to_string())).unwrap();
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

            // enforce the following rules:
            //  #) user can simultaneously be in an interactive room AND a background room; however...
            //  #) user can only be in one interactive room at a time
            //  #) user can only be in one background room at a time
            let peers = PEER_MAP.read().unwrap();
            let peer = peers.get(peer_id).unwrap();

            {
                let rooms = peer.2.rooms.read();
                let rooms = rooms.unwrap();
                // if the user is already in an interactive session, do not allow in
                if room.rtype == RoomType::Interactive.as_str() && rooms[0].is_some() {
                    println!("{} already in interactive room", peer_id);
                    // todo: error handling?
                    // let message = json!({
                    //     "type": "error",
                    //     "rid": rid.clone(),
                    //     "peer_id": peer_id.clone(),
                    //     "message": "only one active interactive room session per pier allowed",
                    // });
                    // peer.1.send(Message::text(message.to_string())).unwrap();
                    return;
                }

                // if the user is already in an interactive session, do not allow in
                if room.rtype == RoomType::Background.as_str() && rooms[1].is_some() {
                    println!("{} already in background room", peer_id);
                    return;
                }
            }

            if room.present.contains(peer_id) {
                println!("{} already in room", peer_id);
                return;
            }

            room.present.push(peer_id.clone());

            let mut slot: i8 = -1;
            // if the user is already in an interactive session, do not allow in
            if room.rtype.as_str() == RoomType::Interactive.as_str() {
                slot = 0;
            } else if room.rtype.as_str() == RoomType::Background.as_str() {
                slot = 1;
            }

            if slot != -1 {
                let rooms = peer.2.rooms.write();
                let mut rooms = rooms.unwrap();
                rooms[slot as usize].replace(());
            }

            // Create the message
            let message = json!({
                "type": "room-entered",
                "rid": rid.clone(),
                "peer_id": peer_id.clone(),
                "room": &room.clone(),
            });

            // send update to all known peers
            // let peers = PEER_MAP.read().unwrap();
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
            let peers = PEER_MAP.read().unwrap();
            // Create the message
            let message = json!({
                "type": "room-left",
                "rid": rid.clone(),
                "peer_id": peer_id.clone(),
                "room": room.clone(),
            });
            // FIX this
            // send update to all known peers
            for (_, (_, sender, _)) in peers.iter() {
                sender.send(Message::text(message.to_string())).unwrap()
            }
            // for peer_id in room.present.iter() {
            //     if let Some((_, sender, _)) = peers.get(peer_id) {
            //         sender.send(Message::text(message.to_string())).unwrap()
            //     }
            // }
        }
        "signal" => {
            // signal_type - webrtc: offer, answer, candidate, renegotiate, transceiverRequest, transceiverAnswer, transceiverIce, transceiverClose
            // signal_type - realm: cursor, chat, file, video, audio, screen
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
            println!("signal_type: {}", signal["type"]);
            let peers = PEER_MAP.read().unwrap();
            if let Some((_, sender, _)) = peers.get(&to) {
                let message = json!({
                    "type": "signal",
                    "rid": rid.clone(),
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
                        // no active background or interactive session by default
                        rooms: Arc::new(RwLock::new([None, None])),
                    },
                ),
            );
        }
        "disconnect" => disconnect(peer_id),
        _ => unknown(),
    };
}

fn unknown() {
    println!("[signaling] unknown message type")
}

fn disconnect(peer_id: &str) {
    println!("{} - disconnect", peer_id);
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
