mod helpers;
use std::convert::Infallible;
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
        .recover(handle_unauthorized);

    warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

    Ok(())
}

#[derive(Debug)]
struct Unauthorized;

impl reject::Reject for Unauthorized {}

#[derive(Debug)]
struct ServerError;

impl reject::Reject for ServerError {}

pub async fn handle_unauthorized(reject: Rejection) -> Result<impl Reply, Rejection> {
    if reject.is_not_found() {
        Ok(warp::redirect(Uri::from_static("/~/login?redirect=/")))
    } else if reject.find::<warp::reject::MissingHeader>().is_some() {
        Ok(warp::redirect(Uri::from_static("/~/login?redirect=/")))
    } else if reject.find::<Unauthorized>().is_some() {
        Ok(warp::redirect(Uri::from_static("/~/login?redirect=/")))
    } else {
        Err(reject)
    }
}

fn check_cookie(
    ship_interface: SafeShipInterface,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(with_ship_interface(ship_interface))
        .and(warp::header::<String>("Cookie"))
        .and_then(
            move |ship_interface: SafeShipInterface, cookie: String| async move {
                let cookie = cookie.split(';').collect::<Vec<&str>>()[0].to_string();
                let res = ship_interface
                    .scry(
                        "holon",
                        format!("/valid-cookie/{}", cookie).as_str(),
                        "json",
                    )
                    .await;
                if res.is_ok() {
                    let is_valid = res
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .get("is-valid")
                        .unwrap()
                        .as_bool()
                        .unwrap();
                    if is_valid {
                        return Ok(());
                    } else {
                        return Err(reject::custom(Unauthorized));
                    }
                } else {
                    match res.err().unwrap() {
                        urbit_api::error::UrbitAPIError::Forbidden => {
                            let res = ship_interface.refresh().await;
                            if res.is_err() {
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
                                return Ok(());
                            } else {
                                return Err(reject::custom(Unauthorized));
                            }
                        }
                        _ => return Err(reject::custom(ServerError)),
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
