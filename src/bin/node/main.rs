mod helpers;
use std::sync::Arc;

use warp::{reject, Filter};

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

    // check for a valid header on ship requests
    let header_check = warp::header::<String>("Cookie")
        .and_then({
            move |cookie: String| {
                let ship_interface = Arc::clone(&ship_interface);
                // split the first ; and take the first part
                let cookie = cookie.split(';').collect::<Vec<&str>>()[0].to_string();
                async move {
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
                }
            }
        })
        .untuple_one();

    let proxy = reverse_proxy_filter("".to_string(), http_server_url);

    let routes = rooms_route.or(signaling_route).or(header_check.and(proxy));

    warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

    Ok(())
}

#[derive(Debug)]
struct Unauthorized;

impl reject::Reject for Unauthorized {}
