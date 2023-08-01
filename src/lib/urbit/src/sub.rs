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
use tokio::time::{sleep, Duration};

use trace::{trace_err_ln, trace_good_ln, trace_info_ln, trace_warn_ln};

pub async fn start(ctx: CallContext) -> Result<()> {
    let receiver = ctx.ship.lock().await.open_channel().await;

    if receiver.is_err() {
        bail!("open_channel call failed");
    }

    let mut receiver = receiver.unwrap();

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

                let err = msg.err();
                if err.unwrap().to_string().contains("403") {
                    // fire a message to all connected devices letting them know about
                    //  the disconnection
                    let msg = json!({
                      "type": "error",
                      "target": "broadcast",
                      "error": "ship-stream-disconnected",
                    });

                    trace_warn_ln!("forwarding error to devices => {}", msg);

                    let send_result = ctx.sender.send(msg);

                    if send_result.is_err() {
                        trace_err_ln!("error sending packet => {:?}", send_result);
                    }

                    receiver = loop {
                        let mut ship = ctx.ship.lock().await;
                        let result = ship.login().await;
                        if result.is_err() {
                            trace_warn_ln!(
                            "login call failed attempting to login after token expiration. trying again in 2 seconds..."
                          );
                            sleep(Duration::from_millis(3000)).await;
                        }
                        let result = ship.open_channel().await;
                        if result.is_err() {
                            trace_warn_ln!(
                              "open_channel call failed attempting to login after token expiration. trying again in 2 seconds..."
                            );
                            sleep(Duration::from_millis(3000)).await;
                        } else {
                            receiver = result.unwrap();
                            break receiver;
                        }
                    }
                }

                continue;
            }

            // the deserialized Event from SSE
            let event = msg.unwrap();

            trace_good_ln!("received event => {}", event);

            let data = serde_json::from_str(&event.data);

            if data.is_err() {
                trace_err_ln!("error deserializing event source message to json");
                continue;
            }

            let data = data.unwrap();

            // log the entire packet to the database
            let _ = ctx.db.save_packet("ship", &data);

            trace_info_ln!("ship: [listen] sending event to receiver => {}", data);

            let send_result = ctx.sender.send(data);

            if send_result.is_err() {
                trace_err_ln!("ship: [listen] error sending packet => {:?}", send_result);
            }
        }
    });

    Ok(())
}
