use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;

use crate::process::get_pid_and_loopback;

pub async fn send_lens_payload(
    server_id: String,
    payload: Value,
) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    let (_, loopback_str) = get_pid_and_loopback(server_id).await?;

    let resp = client
        .post(format!("http://127.0.0.1:{}", loopback_str))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await?;

    if resp.status().is_success() {
        let response: String = resp.json().await.unwrap();
        return Ok(response);
    } else {
        return Err(Box::new(resp.error_for_status().unwrap_err()));
    }
}

pub async fn get_access_code(server_id: String) -> Result<String, Box<dyn Error>> {
    let payload = json!({
        "source": {
            "dojo": "+code"
        },
        "sink": {
            "stdout": null
        }
    });

    let code = send_lens_payload(server_id, payload).await?;

    if !code.is_empty() {
        let code_str = code.to_string();
        return Ok(code_str);
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get access code",
        )));
    }
}
