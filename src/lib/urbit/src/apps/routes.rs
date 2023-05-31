use std::convert::Infallible;
use warp::{self, Filter};

use crate::apps::handlers;
use crate::ShipInterface;

/// All App Store routes
pub fn app_store_routes(
    ship_interface: ShipInterface,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET"]);

    get_app(ship_interface.clone())
        .or(get_apps(ship_interface))
        .with(cors)
}

/// GET /hol/apps
fn get_apps(
    ship_interface: ShipInterface,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("hol" / "apps")
        .and(warp::get())
        .and(with_ship_interface(ship_interface))
        .and_then(handlers::get_apps)
}

/// GET /hol/apps/:desk
fn get_app(
    ship_interface: ShipInterface,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("hol" / "apps" / String)
        .and(warp::get())
        .and(with_ship_interface(ship_interface))
        .and_then(handlers::get_app)
}

fn with_ship_interface(
    ship_interface: ShipInterface,
) -> impl Filter<Extract = (ShipInterface,), Error = Infallible> + Clone {
    warp::any().map(move || ship_interface.clone())
}
