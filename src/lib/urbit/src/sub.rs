///
/// Subscriptions
///
/// For now, there should be one and only one ship subscription listening for events.
///
/// start function: opens a channel to the ship and subscribes to SSE
///   events that come in from the ship are then forwarded to the web socket receiver,
///   where they are ultimately delivered to listening devices over websocket.
///
use crate::context::CallContext;
use anyhow::{bail, Result};

use eventsource_threaded::EventSource;

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};

pub async fn start(ctx: CallContext) -> Result<()> {
    let result = ctx.ship.lock().await.open_channel().await;

    if result.is_err() {
        bail!("sub: [start] open_channel call failed");
    }

    let result = result.unwrap();
    let channel_url = result.0;
    let _ship_name = result.1;
    let session_auth = result.2;

    // Create cookie header with the ship session auth val
    let mut headers = HeaderMap::new();
    headers.append(COOKIE, HeaderValue::from_str(&session_auth)?);

    tokio::spawn(async move {
        let receiver = EventSource::new(channel_url, headers);
        loop {
            println!("ship: [listen] waiting for ship event...");
            let msg = receiver.recv();

            if msg.is_err() {
                println!("ship: [listen] event receive error. msg => {:?}", msg);
                continue;
            }

            let msg = msg.unwrap();

            if msg.is_err() {
                println!("ship: [listen] event receive error. msg =>{:?}", msg);
                continue;
            }

            // the deserialized Event from SSE
            let event = msg.unwrap();

            println!("ship: [listen] received event => {}", event);

            let data = serde_json::from_str(&event.data);

            if data.is_err() {
                println!("ship: [listen] error deserializing event source message to json");
                continue;
            }

            let data = data.unwrap();

            // log the entire packet to the database
            let _ = ctx.db.save_packet("ship", &data);

            println!("ship: [listen] sending event to receiver => {}", data);

            let send_result = ctx.sender.send(data);

            if send_result.is_err() {
                println!("ship: [listen] error sending packet => {:?}", send_result);
            }
        }
    });

    Ok(())
}
