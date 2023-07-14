use anyhow::{bail, Result};
use eventsource_threaded::EventSource;

use rand::Rng;
use reqwest::header::HeaderMap;
use reqwest::Url;
use serde_json::{from_str, json, Value};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

use crate::CallContext;

pub async fn start(ctx: &CallContext) -> Result<()> {
    let mut rng = rand::thread_rng();
    // Defining the uid as UNIX time, or random if error
    let uid = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_micros(),
        Err(_) => rng.gen(),
    }
    .to_string();

    // Channel url
    let channel_url = format!("{}/~/channel/{}", &ctx.ship_interface.get_url().await, uid);
    // Opening channel request json
    let mut body = from_str::<Value>(r#"[]"#).unwrap();
    body[0] = json!({
            "id": 1,
            "action": "poke",
            "ship": ctx.ship_interface.get_ship_name().await,
            "app": "hood",
            "mark": "helm-hi",
            "json": "Opening channel",
    });

    // Make the put request to create the channel.
    let resp = ctx
        .ship_interface
        .send_put_request(&channel_url, &body)
        .await?;

    if resp.status().as_u16() != 204 {
        bail!("chat: [listen] failed to create new channel");
    }
    // Create cookie header with the ship session auth val
    let mut headers = HeaderMap::new();
    headers.append(
        "cookie",
        ctx.ship_interface.get_session_auth().await.unwrap(),
    );
    // Create the receiver
    let url_structured = {
        let url_structured = Url::parse(&channel_url); //.map_err(|_| UrbitAPIError::FailedToCreateNewChannel)?;

        if url_structured.is_err() {
            bail!("chat: [listen] error parsing channel url {}", channel_url)
        }
        url_structured.unwrap()
    };
    let sender = Arc::new(Mutex::new(ctx.sender.clone()));
    tokio::spawn(async move {
        let receiver = EventSource::new(url_structured, headers);
        loop {
            let msg = receiver.recv();
            let input = {
                if msg.is_err() {
                    println!("chat: [listen] event receive error => {:?}", msg);
                    continue;
                }
                let result = msg.unwrap();
                if result.is_err() {
                    println!("chat: [listen] event receive error => {:?}", result);
                    continue;
                }
                let result = result.unwrap();
                println!("chat: [listen] event received => {:?}", result);
                result
            };
            let event_type = {
                if input.event_type.is_none() {
                    return "none";
                }
                input.event_type.unwrap();
            };
            let packet = json!({
              "id": input.id,
              "event_type": event_type,
              "data": input.data,
            });
            let send_result = sender.lock().await.send(packet);
            if send_result.is_err() {
                println!("chat: [listen] error sending packet => {:?}", send_result);
            }
        }
    });
    Ok(())
}
