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
use serde_json::json;

use trace::{trace_err_ln, trace_green_ln, trace_info_ln, trace_warn_ln};

pub async fn start(ctx: CallContext) -> Result<()> {
    let receiver = ctx.ship.lock().await.open_channel().await;

    if receiver.is_err() {
        bail!("sub: [start] open_channel call failed");
    }

    let receiver = receiver.unwrap();

    tokio::spawn(async move {
        loop {
            trace_info_ln!("waiting for ship event...",);

            let msg = receiver.recv();

            if msg.is_err() {
                trace_err_ln!("event receive error. msg => {:?}", msg.err());
                continue;
            }

            let msg = msg.unwrap();

            if msg.is_err() {
                trace_err_ln!("event request error. msg => {:?}", msg);

                let msg = json!({
                  "id": 4884,
                  "type": "error",
                  "error": "ship-stream-disconnected",
                });

                trace_warn_ln!("forwarding error to devices => {}", msg);

                let send_result = ctx.sender.send(msg);

                if send_result.is_err() {
                    trace_err_ln!("error sending packet => {:?}", send_result);
                }

                continue;
            }

            // the deserialized Event from SSE
            let event = msg.unwrap();

            #[cfg(feature = "trace")]
            trace_green_ln!("received event => {}", event);

            let data = serde_json::from_str(&event.data);

            if data.is_err() {
                trace_err_ln!("error deserializing event source message to json");
                continue;
            }

            let data = data.unwrap();

            // log the entire packet to the database
            let _ = ctx.db.save_packet("ship", &data);

            #[cfg(feature = "trace")]
            trace_info_ln!("ship: [listen] sending event to receiver => {}", data);

            let send_result = ctx.sender.send(data);

            if send_result.is_err() {
                trace_err_ln!("ship: [listen] error sending packet => {:?}", send_result);
            }
        }
    });

    Ok(())
}
