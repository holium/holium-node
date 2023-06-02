mod helpers;
use structopt::StructOpt;
use urbit_api::ShipInterface;
use warp::Filter;
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

    // build the globs store from the ships in the app_hosts table
    #[structopt(long = "init-globs")]
    pub init_globs: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = HolAPI::from_args();

    let server_url = format!("127.0.0.1:{}", opt.urbit_port.clone());
    wait_for_server(&server_url.parse().expect("Cannot parse url")).await?;

    // Cannot drop a runtime in a context where blocking is not allowed
    let access_code = urbit_api::lens::get_access_code(opt.server_id.clone()).await?;

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

    let rooms_route = rooms::api::rooms_route();
    let signaling_route = rooms::socket::signaling_route();
    let proxy = reverse_proxy_filter("".to_string(), http_server_url);

    // initialize the glob store (sql lite db) sourced from ships located
    //  in the app_hosts table
    if opt.init_globs {
        urbit_api::apps::store::init(ship_interface.clone()).await?;
    }
    let app_store_routes = urbit_api::apps::routes::app_store_routes(ship_interface);

    warp::path::full().map(|path: warp::path::FullPath| {
        println!("Incoming request at path: {}", path.as_str());
    });

    // let routes = rooms_route.or(proxy);
    let routes = rooms_route
        .or(app_store_routes)
        .or(signaling_route)
        .or(proxy);

    warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

    Ok(())
}
