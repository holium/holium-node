mod helpers;

use std::convert::Infallible;

use serde_derive::Serialize;
use warp::http::uri::PathAndQuery;
use warp::http::StatusCode;
use warp::{http::Uri, reject, Filter, Rejection, Reply};

use structopt::StructOpt;
use urbit_api::SafeShipInterface;

use warp_reverse_proxy::reverse_proxy_filter;

use crate::helpers::wait_for_server;

#[derive(StructOpt)]
pub struct HolAPI {
    #[structopt(name = "hol-api", about = "The webserver part of the node")]

    /// the identity of the instance
    #[structopt()]
    server_id: String,
    /// http-port for Urbit instance
    #[structopt(short = "p", long = "urbit-port", default_value = "9030")]
    pub urbit_port: u16,

    // the port for the Holium node
    #[structopt(long = "node-port", default_value = "3030")]
    pub node_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = HolAPI::from_args();

    let server_url = format!("127.0.0.1:{}", opt.urbit_port.clone());
    wait_for_server(&server_url.parse().expect("Cannot parse url")).await?;

    let access_code = urbit_api::lens::get_access_code(opt.server_id.clone()).await?;

    let http_server_url = format!("http://localhost:{}", opt.urbit_port.clone());

    // set static ship_interface
    let ship_interface: SafeShipInterface =
        SafeShipInterface::new(http_server_url.as_str(), access_code.trim())
            .await
            .expect("Could not create ship interface");

    let scry_res = ship_interface.scry("docket", "/our", "json").await;
    match scry_res {
        Ok(_) => println!("test_scry: {}", scry_res.unwrap().to_string()),
        Err(e) => println!("scry failed: {}", e),
    }

    let rooms_route = rooms::api::rooms_route();
    let signaling_route = rooms::socket::signaling_route();

    let proxy = reverse_proxy_filter("".to_string(), http_server_url);
    let login_route = warp::path!("~" / "login" / ..).and(reverse_proxy_filter(
        "".to_string(),
        format!("http://localhost:{}/~/login/", opt.urbit_port.clone()),
    ));

    let routes = rooms_route.or(signaling_route).or(login_route);

    let routes = routes
        .or(check_cookie(ship_interface).and(proxy))
        .recover(handle_unauthorized)
        .recover(handle_rejection);

    warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

    Ok(())
}

#[derive(Debug)]
struct Unauthorized;

impl reject::Reject for Unauthorized {}

#[derive(Debug)]
struct Redirect {
    pub location: String,
}

impl reject::Reject for Redirect {}

#[derive(Debug)]
struct ServerError;

impl reject::Reject for ServerError {}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_unauthorized(reject: Rejection) -> Result<impl Reply, Rejection> {
    println!("reject: {:?}", reject);
    if reject.is_not_found() {
        Ok(warp::redirect(Uri::from_static("/~/login?redirect=/")))
    } else if let Some(e) = reject.find::<Redirect>() {
        let loc = &e.location;
        let url = format!("/~/login?redirect={loc}").to_string();
        let uri = PathAndQuery::from_maybe_shared(url).unwrap();
        Ok(warp::redirect(Uri::from(uri)))
    } else {
        Err(reject)
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(_) = err.find::<Unauthorized>() {
        code = StatusCode::UNAUTHORIZED;
        message = "UNAUTHORIZED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

// list of paths considered "api" calls; and therefore should return Json data and
//  reject with 401. all other calls (UI calls) should redirect to login
fn reject_on_path(path: &str) -> warp::Rejection {
    match path.starts_with("/~/scry/")
        || path.starts_with("/~/channel/")
        || path.starts_with("/spider/")
    {
        true => {
            println!("unauthorized");
            return reject::custom(Unauthorized);
        }
        false => {
            println!("redirect");
            return reject::custom(Redirect {
                location: format!("{}", path.to_string()),
            });
        }
    }
}

fn check_cookie(
    ship_interface: SafeShipInterface,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::path::full())
        .and(with_ship_interface(ship_interface))
        .and(warp::header::headers_cloned())
        .and_then(
            move |path: warp::path::FullPath,
                  ship_interface: SafeShipInterface,
                  headers: reqwest::header::HeaderMap| async move {
                if !headers.contains_key("Cookie") {
                    println!("no cookie");
                    return Err(reject_on_path(path.as_str()));
                }
                let cookie = headers.get("Cookie").unwrap().to_str().unwrap();
                println!("path: {}, cookie: {}", path.as_str(), cookie);
                let cookie = cookie.split(';').collect::<Vec<&str>>()[0].to_string();
                let res = ship_interface
                    .scry(
                        "holon",
                        format!("/valid-cookie/{}", cookie).as_str(),
                        "json",
                    )
                    .await;
                if res.is_ok() {
                    println!("scry succeeded");
                    let is_valid: bool = res
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .get("is-valid")
                        .unwrap()
                        .as_bool()
                        .unwrap();
                    if is_valid {
                        println!("cookie is valid");
                        return Ok(());
                    } else {
                        println!("cookie is not valid");
                        return Err(reject_on_path(path.as_str()));
                    }
                } else {
                    println!("scry failed");
                    match res.err().unwrap() {
                        urbit_api::error::UrbitAPIError::StatusCode(403) => {
                            println!("forbidden, refreshing...");
                            let res = ship_interface.refresh().await;
                            if res.is_err() {
                                println!("error refreshing");
                                return Err(reject::custom(ServerError));
                            }
                            let res = ship_interface
                                .scry(
                                    "holon",
                                    format!("/valid-cookie/{}", cookie).as_str(),
                                    "json",
                                )
                                .await;
                            if res.is_err() {
                                println!("error scrying after refresh");
                                return Err(reject::custom(ServerError));
                            }
                            let is_valid = res
                                .unwrap()
                                .as_object()
                                .unwrap()
                                .get("is-valid")
                                .unwrap()
                                .as_bool()
                                .unwrap();
                            if is_valid {
                                println!("cookie is valid [post refresh]");
                                return Ok(());
                            } else {
                                println!("cookie is invalid [post refresh]");
                                return Err(reject_on_path(path.as_str()));
                            }
                        }
                        err => {
                            println!("error: {}", err);
                            return Err(reject::custom(ServerError));
                        }
                    }
                }
            },
        )
        .untuple_one()
}

fn with_ship_interface(
    ship_interface: SafeShipInterface,
) -> impl Filter<Extract = (SafeShipInterface,), Error = Infallible> + Clone {
    warp::any().map(move || ship_interface.clone())
}
