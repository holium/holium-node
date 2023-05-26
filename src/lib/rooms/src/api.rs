use warp::Filter;

use crate::types::{Room, ROOM_MAP};

pub fn rooms_route() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let get_rooms = warp::path!("hol" / "rooms")
        .and(warp::get())
        .and_then(|| async { handle_get_session().await });

    get_rooms
}

pub async fn handle_get_session() -> Result<impl warp::Reply, warp::Rejection> {
    let rooms_list = {
        let rooms_state = ROOM_MAP.read().unwrap();
        // convert room map to json
        let mut rooms: Vec<Room> = Vec::new();
        for (_, room) in rooms_state.iter() {
            // convert room to json
            let (_, room_data) = room;
            rooms.push(room_data.write().unwrap().clone());
        }
        rooms
    };
    Ok(warp::reply::json(&rooms_list))
}
