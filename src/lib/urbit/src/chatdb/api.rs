use warp::Filter;

pub fn chat_router() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "DELETE"]);

    let chat_routes = warp::path!("hol" / "chat" / "stats")
        .and(warp::get())
        .and_then(|| async { handle_get_chat_stats().await });

    chat_routes.with(cors)
}

pub async fn handle_get_chat_stats() -> Result<impl warp::Reply, warp::Rejection> {
    // let data = json::object!{};
    Ok(warp::reply::json(&{}))
}
