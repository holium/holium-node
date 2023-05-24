mod helpers;
mod rooms;
use proctitle::set_title;
use structopt::StructOpt;
use urbit_api::ShipInterface;
use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

use crate::{helpers::wait_for_server, rooms::room::ROOMS_STATE};

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

    {
        let mut rooms_state = ROOMS_STATE.lock().unwrap();
        rooms_state.initialize(opt.server_id.clone());
    }

    let server_url = format!("127.0.0.1:{}", opt.urbit_port.clone());
    wait_for_server(&server_url.parse().expect("Cannot parse url")).await?;

    let access_code = helpers::get_access_code(opt.server_id.clone())
        .await
        .expect("Could not get access code");

    let http_server_url = format!("http://localhost:{}", opt.urbit_port.clone());
    let ship_interface = ShipInterface::new(http_server_url.as_str(), access_code.trim())
        .await
        .expect("Could not create ship interface");

    let scry_res = ship_interface.scry("docket", "/our", "json").await.unwrap();
    println!("test_scry: {}", scry_res.text().await.unwrap());

    // let docket_res = ship_interface
    //     .scry("docket", "/charges", "json")
    //     .await
    //     .unwrap();
    // println!("docket_res: {}", docket_res.text().await.unwrap());

    let rooms_route = rooms::rooms_route();

    let proxy = reverse_proxy_filter("".to_string(), http_server_url);

    warp::path::full().map(|path: warp::path::FullPath| {
        println!("Incoming request at path: {}", path.as_str());
    });

    let routes = rooms_route.or(proxy);

    warp::serve(routes)
        .run(([127, 0, 0, 1], opt.node_port))
        .await;

    Ok(())
}
