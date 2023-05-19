use warp::Filter;

use std::collections::HashMap;

use bytes::Bytes;
use reqwest::Client;
use std::convert::Infallible;
use warp::http::{HeaderMap, HeaderValue};
use warp::hyper::Body;
use warp::hyper::Response;
use warp::reply::Response as WarpResponse;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct HolAPI {
    #[structopt(name = "hol-apo", about = "The webserver part of the node")]
    /// the identity of the instance
    // #[structopt()]
    // server_id: String,
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
    let rooms_route = warp::path!("apps" / "rooms-v2").map(|| "This is the apps/rooms-v2 endpoint");

    let client = Client::new();
    let forward = warp::any()
        .and(warp::path::tail())
        .and(warp::header::headers_cloned())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::body::bytes())
        .and_then(
            move |tail: warp::path::Tail,
                  headers: HeaderMap<HeaderValue>,
                  query: HashMap<String, String>,
                  body: Bytes| {
                let client = client.clone();
                let path = tail.as_str();
                println!("{} {} {:?}", path, headers.len(), query.len());
                let query_string: String = query
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                let full_path = if query_string.is_empty() {
                    format!("http://localhost:{}/{}", opt.urbit_port, path)
                } else {
                    format!(
                        "http://localhost:{}/{}?{}",
                        opt.urbit_port, path, query_string
                    )
                };
                async move {
                    let res = if !body.is_empty() {
                        client
                            .post(&full_path)
                            .headers(headers)
                            .body(body.to_vec())
                            .send()
                            .await
                    } else {
                        client.get(&full_path).headers(headers).send().await
                    };
                    let response = res.unwrap();
                    let headers = response.headers().clone();
                    let status = response.status();
                    let bytes = response.bytes().await.unwrap();
                    let mut response_builder = Response::builder().status(status);
                    for (key, value) in headers.iter() {
                        println!("heaer: {}: {:?}", key, value);
                        response_builder = response_builder.header(key, value.clone());
                    }

                    let response = response_builder.body(Body::from(bytes));
                    Ok::<WarpResponse, Infallible>(response.unwrap())
                }
            },
        );

    let routes = rooms_route.or(forward);

    warp::serve(routes)
        .run(([127, 0, 0, 1], opt.node_port))
        .await;
    Ok(())
}
