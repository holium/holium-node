use warp::Filter;

pub fn rooms_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("apps" / "rooms-v2").map(|| warp::reply::html("This is the apps/rooms-v2 endpoint"))
}
