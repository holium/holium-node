use crate::ShipInterface;
use serde_json::{Map, Value};
use warp::Filter;

pub fn app_store_routes(
    ship_interface: ShipInterface,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET"]);

    let get_apps = warp::path!("hol" / "apps")
        .and(warp::get())
        .and_then(|| async { handle_get_apps().await });

    let get_app = warp::path!("hol" / "apps")
        .and(warp::path::param())
        .and_then(|desk: String| async { handle_get_app(desk).await });

    get_apps.with(&cors);
    get_app.with(&cors)
}

pub async fn handle_get_apps() -> Result<impl warp::Reply, warp::Rejection> {
    let apps_list = {
        let mut apps: Option<Map<String, Value>> = None;
        apps
    };
    Ok(warp::reply::json(&apps_list))
}

pub async fn handle_get_app(desk: String) -> Result<impl warp::Reply, warp::Rejection> {
    let app_detail = {
        let mut app: Option<Map<String, Value>> = None;
        app
    };
    Ok(warp::reply::json(&app_detail))
}
