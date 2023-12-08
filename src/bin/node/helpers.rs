use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

use trace::{trace_err_ln, trace_good_ln};

pub async fn wait_for_server(server_addr: &SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match TcpStream::connect(server_addr).await {
            Ok(_) => {
                trace_good_ln!("instance is up");
                return Ok(());
            }
            Err(e) => {
                trace_err_ln!("instance is not up yet: {}", e);
            }
        }
        time::sleep(Duration::from_secs(1)).await;
    }
}
