use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::net::SocketAddr;
use std::process::Command;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

pub async fn get_access_code(server_id: String) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    let payload = json!({
        "source": {
            "dojo": "+code"
        },
        "sink": {
            "stdout": null
        }
    });

    let pid = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "ps aux | grep \"{}_urbit ships\" | grep -v grep | awk '{{print $2}}' | tr -d '\n'",
            server_id
        ))
        .output()
        .expect("failed to find pid");

    let pid_str = String::from_utf8(pid.stdout).unwrap();

    let loopback = Command::new("sh")
        .arg("-c")
        .arg(format!(
          "lsof -n -P -p {} |  grep '127.0.0.1' | grep urbit | grep -i listen | cut -d':' -f2 | cut -c -5 | tr -d '\n'", 
          pid_str
        ))
        .output()
        .expect("failed to find loopback port");

    let loopback_str = String::from_utf8_lossy(&loopback.stdout);

    let resp = client
        .post(format!("http://127.0.0.1:{}", loopback_str))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await?;

    if resp.status().is_success() {
        let code: String = resp.json().await.unwrap();
        let code_str = code.to_string();
        return Ok(code_str);
    } else {
        return Err(Box::new(resp.error_for_status().unwrap_err()));
    }
}

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
