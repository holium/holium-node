use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use warp_real_ip::get_forwarded_for;

use trace::{trace_err_ln, trace_good_ln, trace_info_ln, trace_json_ln, trace_warn_ln};

use crate::types::{PeerId, PeerIp, Room, Session, ROOM_MAP, SESSION_MAP};

// InvalidArgs is the rejection that is raised when the serverId and/or deviceId
//   arguments are missing from the url query string
#[derive(Debug)]
struct InvalidArgs;
impl warp::reject::Reject for InvalidArgs {}

pub fn signaling_route(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let signaling = warp::path("signaling")
        .and(warp::header("user-agent"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(
            /*
              ensure that the url contains the serverId and deviceId as a query string parameter
            */
            |user_agent: String, query_params: HashMap<String, String>| async move {
                trace_info_ln!("user agent: {}", user_agent);

                // deviceId parameter is now required
                let server_id = query_params.get("serverId");

                if server_id.is_none() {
                    trace_err_ln!("invalid args. serverId and deviceId required");
                    return Err(warp::reject::custom(InvalidArgs));
                }

                let session_id = Uuid::new_v4();

                Ok((
                    String::from(session_id.to_string()),
                    String::from(server_id.unwrap()),
                ))
            },
        )
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(get_forwarded_for())
        .map(
            |args: (String, String),
             ws: warp::ws::Ws,
             remote_ip: Option<SocketAddr>,
             peer_ips: Vec<IpAddr>| {
                let peer_ip = match peer_ips.first() {
                    Some(ip) => ip.to_string(),
                    None => remote_ip.unwrap().ip().to_string(),
                };

                trace_info_ln!("upgrading to ws: [{}, {}, {}]", args.0, peer_ip, args.1);

                ws.on_upgrade(move |socket| handle_signaling(socket, peer_ip, args.0, args.1))
            },
        );
    signaling
}

pub async fn handle_signaling(ws: WebSocket, peer_ip: PeerIp, session_id: String, peer_id: String) {
    trace_good_ln!("ws connected: [{}, {}, {}]", session_id, peer_id, peer_ip);

    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (sender, receiver) = mpsc::unbounded_channel();
    let mut receiver = UnboundedReceiverStream::new(receiver);

    let cloned_id = peer_id.clone();
    let cloned_peer_ip = peer_ip.clone();
    let cloned_session_id = session_id.clone();

    tokio::task::spawn(async move {
        while let Some(message) = receiver.next().await {
            let msg: Message = Message::from(message);
            let result = ws_sender.send(msg.clone()).await;

            if result.is_err() {
                let err = result.err().unwrap();
                let msg = msg.to_str().unwrap();
                trace_err_ln!("websocket send error: {}, message={}", err, msg);
                disconnect(
                    cloned_session_id.as_str(),
                    cloned_id.as_str(),
                    cloned_peer_ip.as_str(),
                )
            }
        }
    });

    while let Some(result) = ws_receiver.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                trace_err_ln!("websocket error: {}", e);
                disconnect(session_id.as_str(), peer_id.as_str(), peer_ip.as_str());
                break;
            }
        };
        if let Ok(message) = message.to_str() {
            handle_message(sender.clone(), &session_id, &peer_ip, &peer_id, message).await;
        };
    }
}

pub async fn handle_message(
    sender: UnboundedSender<Message>,
    session_id: &String,
    peer_ip: &PeerIp,
    peer_id: &PeerId,
    message: &str,
) {
    let message: Value = serde_json::from_str(message).expect("Error parsing message");
    match message["type"].as_str().unwrap() {
        // Receive peer info from the client
        "create-room" => {
            print!("create-room: [{}, {}, {}]", session_id, peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let mut rtype = String::from("media");
            if !message["rtype"].is_null() {
                rtype = message["rtype"].as_str().unwrap().to_string();
            }
            // let rtype = message["rtype"].as_str().unwrap().to_string();
            let title = message["title"].as_str().unwrap().to_string();
            println!(". room: '{}'", title);

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
                origin: session_id.to_string(),
                sessions: HashMap::from([(
                    session_id.to_string(),
                    Session {
                        id: session_id.to_string(),
                        peer_id: peer_id.to_string(),
                        peer_ip: peer_ip.to_string(),
                    },
                )]),
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

            trace_info_ln!("create-room...");
            trace_json_ln!(&message);

            // send update to all known peers
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                value.1.send(Message::text(message.to_string())).unwrap()
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

            // @patrick
            //  this is a change based on a request for mobile support. originally
            //  we only sent out the room-created event to the creator of the room
            //  with this change; however, we are going to send the room-created event
            //  to ALL known peers
            // send update to all known peers
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                value.1.send(Message::text(message.to_string())).unwrap()
            }
        }

        "edit-room" => {
            println!("edit-room: [{}, {}, {}]", session_id, peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => {
                    println!(". room not found {}", rid);
                    return;
                }
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
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                value.1.send(Message::text(message.to_string())).unwrap()
            }
        }
        "delete-room" => {
            print!("delete-room: [{}, {}, {}]", session_id, peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            delete_room(session_id, &rid);
        }
        "enter-room" => {
            print!("enter-room: [{}, {}, {}]", session_id, peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();

            // Retrieve the room
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => {
                    println!(". room not found {}", rid);
                    return;
                }
            };

            let mut room = room.write().unwrap();
            println!(". room: '{}'", room.title);

            if room.sessions.contains_key(session_id) {
                println!("{}/{} already in room", session_id, peer_id);
                return;
            }

            if !room.present.contains(peer_id) {
                room.present.push(peer_id.clone());
            }

            // Create the message
            let message = json!({
                "type": "room-entered",
                "rid": rid.clone(),
                "peer_id": peer_id.clone(),
                "room": &room.clone(),
            });

            // send update to all known peers
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                value.1.send(Message::text(message.to_string())).unwrap()
            }
        }
        "leave-room" => {
            print!("leave-room: [{}, {}, {}]", session_id, peer_id, peer_ip);
            let rid = message["rid"].as_str().unwrap().to_string();
            let rooms = ROOM_MAP.read().unwrap();
            let room = match rooms.get(&rid) {
                Some(room) => room,
                None => {
                    println!(". room not found {}", rid);
                    return;
                }
            };
            let mut room = room.write().unwrap();
            println!(". room: '{}'", room.title);

            if room.sessions.contains_key(session_id) {
                room.sessions.remove(session_id);
            }

            if !room
                .sessions
                .iter()
                .any(|(_sid, session)| &session.peer_id == peer_id)
            {
                if let Some(index) = room.present.iter().position(|id| id == peer_id) {
                    room.present.remove(index);
                }
            }

            if room.present.len() == 0 || &room.origin == session_id {
                trace_warn_ln!(
                    "{}/{} is last one in room or the creator. deleting room {}...",
                    session_id,
                    peer_id,
                    rid
                );
                drop(room);
                drop(rooms);
                delete_room(session_id, &rid);
                return;
            }

            // Create the message
            let message = json!({
                "type": "room-left",
                "rid": rid.clone(),
                "peer_id": peer_id.clone(),
                "room": room.clone(),
            });

            // send update to all known peers
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                value.1.send(Message::text(message.to_string())).unwrap()
            }
        }
        "signal" => {
            // signal_type - webrtc: offer, answer, candidate, renegotiate, transceiverRequest, transceiverAnswer, transceiverIce, transceiverClose
            // signal_type - realm: cursor, chat, file, video, audio, screen
            let from = message["from"].as_str().unwrap().to_string();
            let to = message["to"].as_str().unwrap().to_string();
            let rid = message["rid"].as_str().unwrap().to_string();
            // println!(
            //     "{} - signal - {}, {}, {}, {}",
            //     peer_id, peer_ip, from, to, rid
            // );
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

            let message = json!({
                "type": "signal",
                "rid": rid.clone(),
                "from": from,
                "signal": signal,
            });

            // send update to all known peers
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                if value.0.peer_id == to {
                    value.1.send(Message::text(message.to_string())).unwrap()
                }
            }
        }
        "connect" => {
            println!("connect: [{}, {}, {}]", session_id, peer_id, peer_ip);
            let rooms = ROOM_MAP.read().unwrap();
            let rooms: Vec<Room> = rooms
                .iter()
                // .filter(|&(_k, v)| v.read().unwrap().rtype == "interactive")
                .map(|(_, room)| room.read().unwrap().clone())
                .collect();
            let message = json!({
                "type": "rooms",
                "rooms": rooms,
            });

            trace_json_ln!(&message);

            sender.send(Message::text(message.to_string())).unwrap();

            let message = json!({
                "type": "connected",
                "session_id": session_id,
            });

            sender.send(Message::text(message.to_string())).unwrap();

            let mut sessions = SESSION_MAP.write().unwrap();
            sessions.insert(
                session_id.to_string(),
                (
                    Session {
                        id: session_id.to_string(),
                        peer_id: peer_id.to_string(),
                        peer_ip: peer_ip.to_string(),
                    },
                    sender.clone(),
                ),
            );
        }
        "disconnect" => disconnect(session_id, peer_id, peer_ip),
        _ => unknown(),
    };
}

fn unknown() {
    trace_warn_ln!("unknown message type")
}

fn delete_room(session_id: &str, room_id: &str) {
    let mut rooms = ROOM_MAP.write().unwrap();
    let room = match rooms.remove(room_id) {
        Some(room) => room,
        None => {
            println!(". room not found {}", room_id);
            return;
        }
    };

    let room = room.read().unwrap();
    println!(". room: '{}'", room.title);

    let message = json!({
        "type": "room-deleted",
        "rid": room_id.clone(),
    });

    // send update to all known peers
    let sessions = SESSION_MAP.read().unwrap();
    for (_, value) in sessions.iter() {
        value.1.send(Message::text(message.to_string())).unwrap()
    }
}

fn disconnect(session_id: &str, peer_id: &str, peer_ip: &str) {
    println!("disconnect: [{}, {}, {}]", session_id, peer_id, peer_ip);
    let mut room_ids_to_remove = Vec::new();

    {
        let rooms = ROOM_MAP.read().unwrap();
        for (rid, room) in rooms.iter() {
            let mut room = room.write().unwrap();
            if let Some(index) = room
                .sessions
                .iter()
                .position(|(sid, session)| sid == session_id)
            {
                room.present.remove(index);
            }
            // if the peer was the last one in the room or the owner of the room, mark the room for removal
            if room.present.is_empty() || &room.origin == session_id {
                room_ids_to_remove.push(rid.clone());
            }
        }
    }

    println!("deleting these rooms: {:?}", room_ids_to_remove);

    {
        let mut sessions = SESSION_MAP.write().unwrap();
        sessions.remove(session_id);

        // print current peer ids
        println!("Current peers: {:?}", sessions.keys());
        for (_, value) in sessions.iter() {
            println!("[{}, {}, {}]", value.0.id, value.0.peer_id, value.0.peer_ip);
        }
    }

    // Remove rooms in a separate pass to avoid the mutable borrow issue
    {
        let mut rooms = ROOM_MAP.write().unwrap();
        for rid in room_ids_to_remove {
            rooms.remove(&rid);

            let message = json!({
                "type": "room-deleted",
                "rid": rid.clone(),
            });

            // send update to all known peers
            let sessions = SESSION_MAP.read().unwrap();
            for (_, value) in sessions.iter() {
                value.1.send(Message::text(message.to_string())).unwrap()
            }
        }
    }
}
