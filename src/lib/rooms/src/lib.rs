use serde::Deserialize;
use warp::reply::WithStatus;
use warp::Filter;
use warp::{http::StatusCode, reply::Json};

use self::api::{handle_action, handle_get_session, parse_action_type};

pub mod api;
pub mod room;
pub mod state;

#[derive(Debug, Deserialize)]
struct PokeAction {
    // id: i32,
    // action: String,
    ship: String,
    // app: String,
    // mark: String,
    json: serde_json::Value,
}

pub fn parse_poke_json(json: &str) -> (String, String) {
    let records: Vec<PokeAction> = serde_json::from_str(json).unwrap();
    if records.len() == 0 {
        // TODO make this return a 400 error
        return ("zod".to_string(), "[]".to_string());
    }

    return (records[0].ship.to_string(), records[0].json.to_string());
}
pub fn rooms_route() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let post_json_filter = warp::post()
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json());

    let action_route = warp::path!("apps" / "rooms-v2")
        .and(post_json_filter)
        .and_then(handle_action_request);

    let get_session = warp::path!("apps" / "rooms-v2" / "session")
        .and(warp::get())
        .and_then(|| async { handle_get_session().await });

    action_route.or(get_session)
}

async fn handle_action_request(
    body: serde_json::Value,
) -> Result<WithStatus<Json>, warp::Rejection> {
    let json = body.to_string();
    let parsed_action = parse_poke_json(&json);
    let res = match parse_action_type(parsed_action) {
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
