mod helpers;
use std::convert::Infallible;
use std::sync::Arc;
use warp::{http::Uri, reject, Filter, Rejection, Reply};

use structopt::StructOpt;
use urbit_api::ShipInterface;

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
    let ship_interface = Arc::new(
        ShipInterface::new(http_server_url.as_str(), access_code.trim())
            .await
            .expect("Could not create ship interface"),
    );

    let scry_res = ship_interface.scry("docket", "/our", "json").await.unwrap();
    println!("test_scry: {}", scry_res.text().await.unwrap());

    let rooms_route = rooms::api::rooms_route();
    let signaling_route = rooms::socket::signaling_route();

    let proxy = reverse_proxy_filter("".to_string(), http_server_url);
    let login_route = warp::path!("~" / "login" / ..).and(reverse_proxy_filter(
        "".to_string(),
        format!("http://localhost:{}/~/login/", opt.urbit_port.clone()),
    ));

    let routes = rooms_route.or(signaling_route).or(login_route);

    let cookie_exists = warp::header::<String>("Cookie").recover(handle_unauthorized);

    let routes = routes
        .or(cookie_exists)
        .or(check_cookie(ship_interface).and(proxy))
        .recover(handle_unauthorized);

    warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

    Ok(())
}

#[derive(Debug)]
struct Unauthorized;

impl reject::Reject for Unauthorized {}

pub async fn handle_unauthorized(reject: Rejection) -> Result<impl Reply, Rejection> {
    print!("reject: {:?}", reject);
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
    ship_interface: Arc<ShipInterface>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(with_ship_interface(ship_interface))
        .and(warp::header::<String>("Cookie"))
        .and_then(
            move |ship_interface: Arc<ShipInterface>, cookie: String| async move {
                println!("testing cookie: {}", cookie);
                // let ship_interface = Arc::clone(&ship_interface);
                let cookie = cookie.split(';').collect::<Vec<&str>>()[0].to_string();
                let scry_res = ship_interface
                    .scry(
                        "holon",
                        format!("/valid-cookie/{}", cookie).as_str(),
                        "json",
                    )
                    .await
                    .unwrap();

                let is_valid = scry_res
                    .json::<serde_json::Value>()
                    .await
                    .unwrap()
                    .get("is-valid")
                    .unwrap()
                    .as_bool()
                    .unwrap();

                if is_valid {
                    // continue to the proxy
                    Ok(())
                } else {
                    Err(reject::custom(Unauthorized))
                }
            },
        )
        .untuple_one()

    // .untuple_one();
}

fn with_ship_interface(
    ship_interface: Arc<ShipInterface>,
) -> impl Filter<Extract = (Arc<ShipInterface>,), Error = Infallible> + Clone {
    warp::any().map(move || ship_interface.clone())
}
