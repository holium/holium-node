// mod routes;

use std::path::Path;
use std::fs;
use std::process::Child;
use warp::{Filter};

use std::process::Command;
use structopt::StructOpt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use std::convert::Infallible;
use warp::hyper::Body;
use reqwest::Client;
use bytes::Bytes;
use warp::hyper::Response;
use warp::reply::Response as WarpResponse;
use warp::http::{HeaderMap, HeaderValue};


const BINARY_URL: &str = if cfg!(target_os = "macos") {
    "https://urbit.org/install/macos-x86_64/latest"
} else if cfg!(target_os = "linux") {
    "https://urbit.org/install/linux-x86_64/latest"
} else {
    panic!("Unsupported platform");
};

#[derive(StructOpt)]
struct Opt {
    /// The server ID of the Urbit instance
    #[structopt(short="i", long = "server-id")]
    server_id: String,

    /// Start a fake ship
    #[structopt(short="F", long="fake")]
    fake: bool,

    /// The key string of the Urbit instance
    #[structopt(short = "G", long = "key")]
    key: Option<String>, // key is now optional

    /// The port number to run the server on (default: 3030)
    #[structopt(long = "node-port", default_value = "3030")]
    node_port: u16,

    /// The port number to run the server on (default: 3030)
    #[structopt(short = "p", long = "urbit-port", default_value = "9030")]
    urbit_port: u16,

    /// The download path for the Urbit binary (default: urbit)
    #[structopt(short = "b", long = "binary-name", default_value = "urbit")]
    binary_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();


    if !Path::new(&opt.binary_name).exists() {
        println!("Downloading Urbit binary...");
        // Download the latest Urbit binary
        Command::new("curl")
            .arg("-L")
            .arg(BINARY_URL)
            .arg("-o")
            .arg("urbit.tar.gz").status()?;

        // Extract the file
        Command::new("tar")
            .arg("zxvf")
            .arg("urbit.tar.gz")
            .arg("-s")
            .arg("/.*/urbit/")
            .status()?;

        // Make the binary executable
        Command::new("chmod")
            .arg("+x")
            .arg(&opt.binary_name)
            .output()
            .expect("Failed to execute command");

        Command::new("mkdir").arg("ships").output().expect("Failed to execute command");

        // remove the tar file
        fs::remove_file("urbit.tar.gz")?;
    }

    println!("Starting urbit...");
    let mut command = Command::new("./urbit");
            
    // check if a folder with the server ID exists
    if !Path::new(&opt.server_id).exists() {
        if opt.fake {
            command.arg("-F");
            command.arg(&opt.server_id.to_string());
        } else if let Some(key) = &opt.key {
            command.arg("-w").arg(&opt.server_id);
            command.arg("-G").arg(key);
        }
    } 
    
    command.arg("-c").arg(format!("ships/{}", &opt.server_id))
        .arg("--http-port").arg(&opt.urbit_port.to_string());

    if Path::new(&opt.server_id).exists() {
        command.arg(&opt.server_id);
    }

    println!("Starting urbit with args: {:?}", command.get_args().collect::<Vec<_>>());
    let child_processes: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));

    let child = command.spawn().expect("Failed to start Urbit instance");
    child_processes.lock().unwrap().insert(opt.server_id.clone(), child);
    println!("Started Urbit instance with PID {}", child_processes.lock().unwrap()[&opt.server_id].id());

    let child_processes_clone = Arc::clone(&child_processes);

    ctrlc::set_handler(move || {
        let child_processes_clone = Arc::clone(&child_processes_clone);
        let server_id = opt.server_id.clone();
        let server_id_str = server_id.clone();
        let child_id;
        {
            let mut instances = child_processes_clone.lock().unwrap();
            if let Some(child) = instances.get(&server_id_str) {
                child_id = child.id().to_string();
                instances.remove(&server_id_str);
            } else {
                println!("No Urbit instance with server ID {} found \n", server_id_str);
                return;
            }
        }
        
        let _ = std::process::Command::new("kill")
            .arg("-9")
            .arg(child_id)
            .output();

        println!("\nUrbit instance with server ID {} stopped \n", server_id_str);

        // Exiting the program
        std::process::exit(0);

    }).expect("Error setting Ctrl-C handler");


    let child_processes_clone = Arc::clone(&child_processes);
    let get_instances = warp::get()
        .and(warp::path("instances"))
        .and_then(move || {
            let child_processes_clone = Arc::clone(&child_processes_clone);
            async move {
                let mut response = HashMap::new();
                {
                    let instances = child_processes_clone.lock().unwrap();
                    for (server_id, child) in instances.iter() {
                        response.insert(server_id.clone(), child.id());
                    }
                }
                Ok::<_, warp::Rejection>(warp::reply::json(&response))
            }
        });
    let urbit_port = opt.urbit_port;
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
                format!("http://localhost:{}/{}", urbit_port, path)
            } else {
                format!("http://localhost:{}/{}?{}", urbit_port, path, query_string)
            };
            async move {
                let res = if !body.is_empty() {
                    client.put(&full_path).headers(headers).body(body.to_vec()).send().await
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


    let routes = get_instances.or(forward);
    warp::serve(routes).run(([127, 0, 0, 1], opt.node_port)).await;
    Ok(())
}
