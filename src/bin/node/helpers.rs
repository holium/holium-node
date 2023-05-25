use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

pub async fn wait_for_server(server_addr: &SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match TcpStream::connect(server_addr).await {
            Ok(_) => {
                println!("instance is up");
                return Ok(());
            }
            Err(e) => {
                println!("instance is not up yet: {}", e);
            }
        }
        time::sleep(Duration::from_secs(1)).await;
    }
}
