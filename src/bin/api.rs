use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

use structopt::StructOpt;

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

    let proxy_filter = reverse_proxy_filter(
        "".to_string(),
        format!("http://localhost:{}", opt.urbit_port),
    );

    let rooms_route = warp::path!("apps" / "rooms-v2").map(|| "This is the apps/rooms-v2 endpoint");

    let routes = rooms_route.or(proxy_filter);

    warp::serve(routes)
        .run(([127, 0, 0, 1], opt.node_port))
        .await;

    Ok(())
}
