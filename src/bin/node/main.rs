mod rooms;
// mod socket;
use structopt::StructOpt;
use urbit_api::ShipInterface;
use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

use crate::rooms::room::ROOMS_STATE;

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
    let server_url = format!("http://0.0.0.0:{}", opt.urbit_port.clone());

    println!("Starting Holium node on port {}", opt.urbit_port);

    {
        let mut rooms_state = ROOMS_STATE.lock().unwrap();
        rooms_state.initialize(opt.server_id.clone());
    }

    let ship_interface = ShipInterface::new(server_url.as_str(), "lidlut-tabwed-pillex-ridrup")
        .await
        .unwrap();

    let scry_res = ship_interface.scry("docket", "/our", "json").await.unwrap();
    println!("scry_res: {}", scry_res.text().await.unwrap());

    let docket_res = ship_interface
        .scry("docket", "/charges", "json")
        .await
        .unwrap();
    println!("docket_res: {}", docket_res.text().await.unwrap());

    let rooms_route = rooms::rooms_route();

    let proxy = reverse_proxy_filter("".to_string(), server_url);

    warp::path::full().map(|path: warp::path::FullPath| {
        println!("Incoming request at path: {}", path.as_str());
    });

    let routes = rooms_route.or(proxy);

    warp::serve(routes)
        .run(([127, 0, 0, 1], opt.node_port))
        .await;

    Ok(())
}

// let socket_map: Arc<RwLock<HashMap<String, Mutex<WebSocket>>>> =
//     Arc::new(RwLock::new(HashMap::new()));
// let queued_signals: Arc<RwLock<HashMap<String, Vec<SocketData>>>> =
//     Arc::new(RwLock::new(HashMap::new()));

// let socket_route = socket::socket_route(Arc::clone(&socket_map), Arc::clone(&queued_signals));
