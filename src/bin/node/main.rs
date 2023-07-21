mod helpers;

use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;

use serde_derive::Serialize;
use serde_json::Value as JsonValue;
use warp::http::uri::PathAndQuery;
use warp::http::StatusCode;
use warp::{http::Uri, reject, Filter, Rejection, Reply};

use tokio::sync::mpsc::unbounded_channel;

use structopt::StructOpt;
use urbit_api::api::Ship;

use warp_reverse_proxy::reverse_proxy_filter;

use crate::helpers::wait_for_server;

use urbit_api::context::{CallContext, NodeContext};
use urbit_api::db::Db;

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

    let mut ship: Ship = Ship::new(http_server_url.as_str(), access_code.trim())
        .await
        .expect("Could not create ship interface");

    let scry_res = ship.scry("docket", "/our", "json").await;
    match scry_res {
        Ok(_) => println!("test_scry: {}", scry_res.unwrap().to_string()),
        Err(e) => println!("scry failed: {}", e),
    }

    // create a new database file (bedrock.sqlite) in the ./src/lib/db/data folder
    let db_pool = bedrock_db::initialize_pool("bedrock")?;

    let (sender, receiver) = unbounded_channel::<JsonValue>();

    // create a call context that is used as a sort of global state for shared instances
    let context: CallContext = NodeContext::to_call_context(NodeContext {
        db: Db { pool: db_pool },
        ship: Arc::new(Mutex::new(ship)),
        // used to send data from the EventSource (task/thread/loop) to the receiver
        sender: sender,
        // threaded listener that waits for messages dispatched by the sender thread
        //  note: need to wrap in Arc::Mutex since will need a mutable reference from within
        //  the leveraging thread (see ws.rs)
        receiver: Arc::new(Mutex::new(receiver)),
    });

    //
    // start each 'module'
    //  panic if any of these fail?
    //

    // start the chat 'module'
    urbit_api::chat::core::start(&context).await?;

    //
    // note:
    // if websockets or ship subscription fails, the process should not start
    //

    // setup the websocket 'hub' which listens for new packets from ctx.receiver
    //  and transmits the events to all client subscribers to the socket
    let ws_route = urbit_api::ws::start(context.clone()).await;

    // subscribe to the ship and listen for events/updates
    // note: clones of Arc are not "expensive", since they only increase the
    // reference count to the underlying data, but do not allocate any new
    // memory nor copy values, etc.
    let res = urbit_api::sub::start(context.clone()).await;

    if res.is_err() {
        panic!("main: [main] error starting ship subscription");
    }

    let rooms_route = rooms::api::rooms_route();
    let signaling_route = rooms::socket::signaling_route();
    let chat_route = urbit_api::chat::api::chat_router(context.clone());

    let proxy = reverse_proxy_filter("".to_string(), http_server_url);
    let login_route = warp::path!("~" / "login" / ..).and(reverse_proxy_filter(
        "".to_string(),
        format!("http://localhost:{}/~/login/", opt.urbit_port.clone()),
    ));

    let routes = rooms_route
        .or(signaling_route)
        // .or(ws_route)
        .or(chat_route)
        .or(login_route);

    let routes = ws_route
        .or(routes)
        .or(check_cookie(context).and(proxy))
        .recover(handle_unauthorized)
        .recover(handle_rejection);
    warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

    Ok(())
}
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let opt = HolAPI::from_args();

//     let server_url = format!("127.0.0.1:{}", opt.urbit_port.clone());
//     wait_for_server(&server_url.parse().expect("Cannot parse url")).await?;

//     let access_code = urbit_api::lens::get_access_code(opt.server_id.clone()).await?;

//     let http_server_url = format!("http://localhost:{}", opt.urbit_port.clone());

//     // set static ship_interface
//     // let ship: Ship = Ship::new(http_server_url.as_str(), access_code.trim())
//     //     .await
//     //     .expect("Could not create ship interface");

//     // let scry_res = ship_interface.scry("docket", "/our", "json").await;
//     // match scry_res {
//     //     Ok(_) => println!("test_scry: {}", scry_res.unwrap().to_string()),
//     //     Err(e) => println!("scry failed: {}", e),
//     // }

//     // create a new database file (bedrock.sqlite) in the ./src/lib/db/data folder
//     let db = bedrock_db::db::initialize("bedrock")?;

//     let (sender, receiver) = unbounded_channel::<JsonValue>();

//     // create a call context that is used as a sort of global state for shared instances
//     let context: CallContext = NodeContext::to_call_context(NodeContext {
//         db: db,
//         // ship: ship,
//         // used to send data from the EventSource (task/thread/loop) to the receiver
//         sender: sender,
//         // threaded listener that waits for messages dispatched by the sender thread
//         //  note: need to wrap in Arc::Mutex since will need a mutable reference from within
//         //  the leveraging thread (see ws.rs)
//         receiver: Arc::new(Mutex::new(receiver)),
//     });

//     //
//     // start each 'module'
//     //  panic if any of these fail?
//     //

//     // start the chat 'module'
//     // urbit_api::chat::core::start(&context).await?;

//     //
//     // note:
//     // if websockets or ship subscription fails, the process should not start
//     //

//     // setup the websocket 'hub' which listens for new packets from ctx.receiver
//     //  and transmits the events to all client subscribers to the socket
//     // let ws_route = urbit_api::ws::start(context.clone());

//     // subscribe to the ship and listen for events/updates
//     // note: clones of Arc are not "expensive", since they only increase the
//     // reference count to the underlying data, but do not allocate any new
//     // memory nor copy values, etc.
//     // let res = urbit_api::sub::start(context.clone()).await;

//     if res.is_err() {
//         panic!("main: [main] error starting ship subscription");
//     }

//     let rooms_route = rooms::api::rooms_route();
//     let signaling_route = rooms::socket::signaling_route();
//     let chat_route = urbit_api::chat::api::chat_router(ctx);

//     let proxy = reverse_proxy_filter("".to_string(), http_server_url);
//     let login_route = warp::path!("~" / "login" / ..).and(reverse_proxy_filter(
//         "".to_string(),
//         format!("http://localhost:{}/~/login/", opt.urbit_port.clone()),
//     ));

//     let routes = rooms_route
//         .or(signaling_route)
//         // .or(ws_route)
//         .or(chat_route)
//         .or(login_route);

//     let routes = ws_route
//         .or(routes)
//         .or(check_cookie(ship_interface).and(proxy))
//         .recover(handle_unauthorized)
//         .recover(handle_rejection);
//     warp::serve(routes).run(([0, 0, 0, 0], opt.node_port)).await;

//     Ok(())
// }

#[derive(Debug)]
struct Unauthorized;

impl reject::Reject for Unauthorized {}

#[derive(Debug)]
struct Redirect {
    pub location: String,
}

impl reject::Reject for Redirect {}

#[derive(Debug)]
struct ServerError;

impl reject::Reject for ServerError {}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_unauthorized(reject: Rejection) -> Result<impl Reply, Rejection> {
    if cfg!(feature = "debug_log") {
        println!("handle_unauthorized: {:?}", reject);
    }

    if reject.is_not_found() {
        Ok(warp::redirect(Uri::from_static("/~/login?redirect=/")))
    } else if let Some(e) = reject.find::<Redirect>() {
        let loc = &e.location;
        let url = format!("/~/login?redirect={loc}").to_string();
        let uri = PathAndQuery::from_maybe_shared(url).unwrap();
        Ok(warp::redirect(Uri::from(uri)))
    } else {
        Err(reject)
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if cfg!(feature = "debug_log") {
        println!("handle_rejection: {:?}", err);
    }

    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(_) = err.find::<Unauthorized>() {
        code = StatusCode::FORBIDDEN;
        message = "FORBIDDEN";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

// list of paths considered "api" calls; and therefore should return Json data and
//  reject with 401. all other calls (UI calls) should redirect to login
fn reject_on_path(path: &str) -> warp::Rejection {
    match path.starts_with("/~/scry/")
        || path.starts_with("/~/channel/")
        || path.starts_with("/spider/")
    {
        true => {
            return reject::custom(Unauthorized);
        }
        false => {
            return reject::custom(Redirect {
                location: format!("{}", path.to_string()),
            });
        }
    }
}

fn handle_response(path: &str, data: JsonValue) -> Result<(), warp::Rejection> {
    let is_valid = data
        .as_object()
        .unwrap()
        .get("is-valid")
        .unwrap()
        .as_bool()
        .unwrap();
    if is_valid {
        if cfg!(feature = "debug_log") {
            println!("cookie valid {}", path)
        }
        return Ok(());
    } else {
        if cfg!(feature = "debug_log") {
            println!("cookie invalid {}", path)
        }
        return Err(reject_on_path(path));
    }
}

fn check_cookie(ctx: CallContext) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::path::full())
        .and(with_call_context(ctx))
        .and(warp::header::headers_cloned())
        .and_then(
            move |path: warp::path::FullPath,
                  context: CallContext,
                  headers: reqwest::header::HeaderMap| async move {
                if !headers.contains_key("Cookie") {
                    return Err(reject_on_path(path.as_str()));
                }
                let cookie = headers.get("Cookie").unwrap().to_str().unwrap();
                if cfg!(feature = "debug_log") {
                    println!("path: {}, cookie: {}", path.as_str(), cookie);
                }
                let cookie = cookie.split(';').collect::<Vec<&str>>()[0].to_string();
                let res = context
                    .ship
                    .lock()
                    .await
                    .scry(
                        "holon",
                        format!("/valid-cookie/{}", cookie).as_str(),
                        "json",
                    )
                    .await;
                if res.is_err() {
                    return Err(reject::custom(ServerError));
                }
                return handle_response(path.as_str(), res.unwrap());
            },
        )
        .untuple_one()
}

fn with_call_context(
    context: CallContext,
) -> impl Filter<Extract = (CallContext,), Error = Infallible> + Clone {
    warp::any().map(move || context.clone())
}
