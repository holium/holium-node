use crate::context::CallContext;
use anyhow::{bail, Result};
use serde_json::json;

use eventsource_threaded::EventSource;

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};

pub async fn start(ctx: CallContext) -> Result<()> {
    let result = ctx.ship.open_channel().await;

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

            let input = {
                if msg.is_err() {
                    println!("ship: [listen] event receive error. msg => {:?}", msg);
                    continue;
                }
                let result = msg.unwrap();
                if result.is_err() {
                    println!("ship: [listen] event receive error. result =>{:?}", result);
                    continue;
                }
                result.unwrap()
            };

            let event_type = 'event_type: {
                if input.event_type.is_none() {
                    break 'event_type String::from("none");
                }
                input.event_type.unwrap()
            };

            let packet = json!({
              "id": input.id,
              "event_type": event_type,
              "data": input.data
            });

            println!("ship: [listen] sending event to receiver => {}", packet);

            let _ = ctx.db.save_packet("ship", &packet);

            let send_result = ctx.sender.send(packet);

            if send_result.is_err() {
                println!("ship: [listen] error sending packet => {:?}", send_result);
            }
        }
    });

    Ok(())
}
