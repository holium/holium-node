use warp::reply::WithStatus;
use warp::Filter;
use warp::{http::StatusCode, reply::Json};

use self::api::{handle_action, parse_action_type};

pub mod api;
pub mod room;

pub fn rooms_route() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let post_json_filter = warp::post()
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json());

    warp::path!("apps" / "rooms-v2")
        .and(post_json_filter)
        .and_then(handle_request)
}

pub async fn handle_request(body: serde_json::Value) -> Result<WithStatus<Json>, warp::Rejection> {
    let json = body.to_string();
    let res = match parse_action_type(&json) {
        Ok(action) => {
            handle_action(action).await?;
            ("Event handled", StatusCode::OK)
        }
        Err(e) => {
            eprintln!("Failed to parse action type: {}", e);
            ("Failed to parse action type", StatusCode::BAD_REQUEST)
        }
    };
    Ok(warp::reply::with_status(warp::reply::json(&res.0), res.1))
}
