pub mod cli;
pub mod urbit;
pub mod api;

use warp::{Filter};

use structopt::StructOpt;
use std::collections::HashMap;
// use std::sync::{Arc};

use std::convert::Infallible;
use warp::hyper::Body;
use reqwest::Client;
use bytes::Bytes;
use warp::hyper::Response;
use warp::reply::Response as WarpResponse;
use warp::http::{HeaderMap, HeaderValue};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = cli::Hol::from_args();

    cli::start(cli::Hol::from_args()).await?;
    // api::start()

    let client = Client::new();
    let forward = warp::any()
        .and(warp::path::tail())
        .and(warp::header::headers_cloned())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::body::bytes())
        .and_then(move |tail: warp::path::Tail, headers: HeaderMap<HeaderValue>, query: HashMap<String, String>, body: Bytes| {
            let client = client.clone();
            let path = tail.as_str();
            let query_string: String = query.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&");
            let full_path = if query_string.is_empty() {
                format!("http://localhost:{}/{}", opt.urbit_port, path)
            } else {
                format!("http://localhost:{}/{}?{}", opt.urbit_port, path, query_string)
            };
            async move {
                let res = if !body.is_empty() {
                    client.post(&full_path).headers(headers).body(body.to_vec()).send().await
                } else {
                    client.get(&full_path).headers(headers).send().await
                };
                let response = res.unwrap();
                let status = response.status();
                let bytes = response.bytes().await.unwrap();
                let response = Response::builder()
                    .status(status)
                    .body(Body::from(bytes));
                Ok::<WarpResponse, Infallible>(response.unwrap())
            }
        });


    let routes = forward;
    warp::serve(routes).run(([127, 0, 0, 1], opt.node_port)).await;
    Ok(())
}
