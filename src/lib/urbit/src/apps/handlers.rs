// Thin wrapper to the underying app store data
// Split out this way so that store can be called from other crates, modules, etc...
//   e.g. in support of the CLI commands vs. REST API

use std::convert::Infallible;
use warp::{self, http::StatusCode};

use crate::apps::store;
use crate::ShipInterface;

pub async fn get_apps(ship_interface: ShipInterface) -> Result<Box<dyn warp::Reply>, Infallible> {
    match store::get_apps(ship_interface).await {
        Ok(apps) => return Ok(Box::new(warp::reply::json(&apps))),
        Err(error) => match error {
            store::AppStoreAPIError::MissingAppData => return Ok(Box::new(StatusCode::NOT_FOUND)),
            store::AppStoreAPIError::AppNotFound => return Ok(Box::new(StatusCode::NOT_FOUND)),
            _ => return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR)),
        },
    };
}

pub async fn get_app(
    desk: String,
    ship_interface: ShipInterface,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    match store::get_app_detail(ship_interface, &desk).await {
        Ok(app) => return Ok(Box::new(warp::reply::json(&app))),
        Err(error) => match error {
            store::AppStoreAPIError::MissingAppData => return Ok(Box::new(StatusCode::NOT_FOUND)),
            store::AppStoreAPIError::AppNotFound => return Ok(Box::new(StatusCode::NOT_FOUND)),
            _ => return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR)),
        },
    };
}
