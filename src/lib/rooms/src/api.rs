// use serde_json::{json, Value as JsonValue};
use warp::Filter;

use crate::types::{Room, Session, ROOM_MAP, SESSION_MAP};

pub fn rooms_route() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "DELETE"]);

    let get_rooms = warp::path!("hol" / "rooms" / ..)
        .and(warp::get())
        .and(
            warp::path::param::<String>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) }),
        )
        .and_then(|arg: Option<String>| async { handle_get_session(arg).await });

    let get_peers = warp::path!("hol" / "sessions" / ..)
        .and(warp::get())
        .and(
            warp::path::param::<String>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) }),
        )
        .and_then(|arg: Option<String>| async { handle_get_peers(arg).await });

    get_rooms.or(get_peers).with(cors)
}

pub async fn handle_get_session(arg: Option<String>) -> Result<impl warp::Reply, warp::Rejection> {
    let arg = arg.as_ref();
    let rooms_list = {
        let rooms_state = ROOM_MAP.read().unwrap();
        // convert room map to json
        let mut rooms: Vec<Room> = Vec::new();
        for room in rooms_state.iter() {
            // convert room to json
            let (_, room_data) = room;

            // by default, only return "room" rooms; otherwise allow additional types
            if arg.is_none() || (arg.is_some() && arg.unwrap() == "all") {
                rooms.push(room_data.write().unwrap().clone());
            } else if &room.1.read().unwrap().rtype == arg.unwrap() {
                rooms.push(room_data.write().unwrap().clone());
            }
        }
        rooms
    };
    Ok(warp::reply::json(&rooms_list))
}

pub async fn handle_get_peers(arg: Option<String>) -> Result<impl warp::Reply, warp::Rejection> {
    let _arg = arg.as_ref();

    let mut result: Vec<Session> = Vec::new();
    // send update to all known peers
    let sessions = SESSION_MAP.read().unwrap();
    for (_, value) in sessions.iter() {
        result.push(value.0.clone());
    }

    Ok(warp::reply::json(&result))
}
