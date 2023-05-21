mod rooms;
// mod socket;

use structopt::StructOpt;
use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

#[derive(StructOpt)]
pub struct HolAPI {
    #[structopt(name = "hol-api", about = "The webserver part of the node")]

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

    // Establish a connection pool.
    // let pool = match establish_connection_pool("my_database.db") {
    //     Ok(pool) => pool,
    //     Err(e) => {
    //         eprintln!("Failed to create connection pool: {}", e);
    //         std::process::exit(1);
    //     }
    // };

    let rooms_route = rooms::rooms_route();

    let proxy = reverse_proxy_filter(
        "".to_string(),
        format!("http://localhost:{}", opt.urbit_port),
    );

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
