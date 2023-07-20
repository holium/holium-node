use anyhow::{bail, Result};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use reqwest::header::{HeaderValue, COOKIE};
use reqwest::Url;
use reqwest::{Client, Response, StatusCode};

use crate::error::{Result as UrbitResult, UrbitAPIError};
use rand::Rng;

use eventsource_threaded::EventSource;

use reqwest::header::HeaderMap;

use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct Ship {
    /// The URL of the ship given as `http://ip:port` such as
    /// `http://0.0.0.0:8080`.
    pub url: String,
    // ship code
    ship_code: String,
    /// The session auth string header value
    pub session_auth: Arc<RwLock<Option<String>>>,
    /// The ship name (without a leading ~)
    pub ship_name: Arc<RwLock<Option<String>>>,
    /// The Reqwest `Client` to be reused for making requests
    req_client: Client,
}

impl Ship {
    pub async fn new(url: &str, ship_code: &str) -> Result<Ship> {
        let result = Ship {
            url: url.to_string(),
            session_auth: Arc::new(RwLock::new(None)),
            ship_name: Arc::new(RwLock::new(None)),
            req_client: Client::new(),
            ship_code: ship_code.to_string(),
        };
        match result.login().await {
            Ok(_) => Ok(result),
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn login(&self) -> Result<(String, String)> {
        let login_url = format!("{}/~/login", self.url);
        let resp = self
            .req_client
            .post(&login_url)
            .body("password=".to_string() + &self.ship_code)
            .send()
            .await?;

        // Check for status code
        if resp.status().as_u16() != 204 {
            bail!(
                "ship: [login] login failed. status code = {}",
                resp.status().as_u16()
            )
        }

        // Acquire the session auth header value
        let session_auth = resp.headers().get("set-cookie");

        if session_auth.is_none() {
            bail!("ship: [login] login failed. set-cookie header not found")
        }

        let session_auth = session_auth.unwrap().to_str();
        if session_auth.is_err() {
            bail!("ship: [login] login failed. unable to get string value from HeaderValue")
        }

        let session_auth = session_auth.unwrap();

        // Trim the auth string to acquire the ship name
        let end_pos = {
            let end_pos = session_auth.find('=');
            if end_pos.is_none() {
                bail!("ship: [login] login failed. invalid cookie returned from ship")
            }
            end_pos.unwrap()
        };

        let ship_name = &session_auth[9..end_pos];

        // safely updated the instance variables with the new values
        self.ship_name.write().await.replace(ship_name.to_string());
        self.session_auth
            .write()
            .await
            .replace(session_auth.to_string());

        Ok((ship_name.to_string(), session_auth.to_string()))
    }

    pub async fn start_listener(&self, sender: UnboundedSender<JsonValue>) -> Result<JsonValue> {
        let mut rng = rand::thread_rng();
        // Defining the uid as UNIX time, or random if error
        let uid = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_micros(),
            Err(_) => rng.gen(),
        }
        .to_string();

        // Channel url
        let channel_url = format!("{}/~/channel/{}", &self.url, uid);

        let session_auth = { self.session_auth.read().await };
        let session_auth = session_auth.as_ref().unwrap();

        let ship_name = { self.ship_name.read().await };
        let ship_name = ship_name.as_ref().unwrap();

        // Opening channel request json
        let body = json!([{
                "id": 1,
                "action": "poke",
                "ship": ship_name,
                "app": "hood",
                "mark": "helm-hi",
                "json": "Opening channel",
        }]);

        // Make the put request to create the channel.
        let resp = self
            .send_put_request(&channel_url, session_auth, &body)
            .await;

        if resp.is_err() {
            bail!("ship: [start_listener] failed to open channel. put request failed.");
        }

        let resp = resp.unwrap();

        if resp.status().as_u16() != 200 {
            bail!(
                "ship: [start_listener] failed to open channel. {}",
                resp.status().as_u16()
            );
        }

        // this is what actually gets returned to the caller
        let return_value = resp.json().await?;

        // Create cookie header with the ship session auth val
        let mut headers = HeaderMap::new();
        headers.append("cookie", HeaderValue::from_str(session_auth)?);

        // Create the receiver
        let url_structured = {
            let url_structured = Url::parse(&channel_url); //.map_err(|_| UrbitAPIError::FailedToCreateNewChannel)?;

            if url_structured.is_err() {
                bail!("chat: [listen] error parsing channel url {}", channel_url)
            }

            url_structured.unwrap()
        };

        tokio::spawn(async move {
            let receiver = EventSource::new(url_structured, headers);
            loop {
                println!("chat: [listen] waiting for ship event...");

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
                    if cfg!(feature = "debug_log") {
                        println!("chat: [listen] event received => {:?}", result);
                    }
                    result
                };

                let event_type = 'event_type: {
                    if input.event_type.is_none() {
                        break 'event_type String::from("none");
                    }
                    input.event_type.unwrap()
                };

                if cfg!(feature = "debug_log") {
                    println!("chat: [listen] sending event to receiver...");
                }

                let packet = json!({
                  "id": input.id,
                  "event_type": event_type,
                  "data": input.data
                });

                let send_result = sender.send(packet);

                if send_result.is_err() {
                    println!("chat: [listen] error sending packet => {:?}", send_result);
                }
            }
        });

        Ok(return_value)
    }

    // Send a put request using the `ShipInterface`
    pub async fn send_put_request(
        &self,
        url: &str,
        session_auth: &str,
        body: &JsonValue,
    ) -> Result<Response> {
        let json = body.to_string();
        let resp = self
            .req_client
            .put(url)
            .header(COOKIE, session_auth)
            .header("Content-Type", "application/json")
            .body(json);

        Ok(resp.send().await?)
    }

    /// Sends a scry to the ship
    pub async fn scry(&self, app: &str, path: &str, mark: &str) -> UrbitResult<JsonValue> {
        let scry_url = format!("{}/~/scry/{}{}.{}", self.url, app, path, mark);
        let session_auth = { self.session_auth.read().await };
        let session_auth = session_auth.as_ref().unwrap();
        let ship_response_as_json: JsonValue = 'response_json: {
            let resp = self
                .req_client
                .get(&scry_url)
                .header(COOKIE, session_auth)
                .header("Content-Type", "application/json");
            let result = resp.send().await?;
            if result.status().as_u16() == StatusCode::FORBIDDEN {
                println!("ship: [scry] session expired. logging in...");
                let result = self.login().await;
                if result.is_err() {
                    println!("ship: [scry] login failed");
                    return Err(UrbitAPIError::FailedToLogin);
                }
                let resp = self
                    .req_client
                    .get(&scry_url)
                    .header(COOKIE, session_auth)
                    .header("Content-Type", "application/json");
                let result = resp.send().await?;
                if result.status().as_u16() != 200 {
                    println!(
                        "ship/api: [scry] retry failed. error {}",
                        result.status().as_u16()
                    );
                    return Err(UrbitAPIError::StatusCode(result.status().as_u16()));
                }
                break 'response_json result.json().await.unwrap();
            }
            if result.status() != 204 {
                println!("ship: [post] failed to post payload");
                return Err(UrbitAPIError::StatusCode(result.status().as_u16()));
            }
            result.json().await.unwrap()
        };
        Ok(ship_response_as_json)
    }

    // use this method to forward actions posted over web socket connection
    //   originating from connected devices
    // this method will attempt to refresh the urbit auth cookie if the
    //   request fails with a 403 (forbidden).
    pub async fn post(&self, payload: &JsonValue) -> Result<JsonValue> {
        let session_auth = self.session_auth.read().await;
        if session_auth.is_none() {
            bail!("ship: [post] must call login");
        }
        let session_auth = session_auth.as_ref().unwrap();
        let ship_response_as_json: JsonValue = 'response_json: {
            let res = self
                .req_client
                .post(&self.url)
                .header(COOKIE, session_auth)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await?;
            // if it's a 403, this indicates the auth header is invalid or expired
            //  try to fetch another token and retry
            if res.status() == StatusCode::FORBIDDEN {
                let result = self.login().await;
                if result.is_err() {
                    bail!("ship: [post] login failed")
                }
                let res = self
                    .req_client
                    .post(&self.url)
                    .header(COOKIE, session_auth)
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;
                if res.status() != 204 {
                    bail!("ship: [post] retry failed")
                }
                break 'response_json res.json().await.unwrap();
            }
            if res.status() != 204 {
                bail!("ship: [post] failed to post payload")
            }
            res.json().await.unwrap()
        };
        Ok(ship_response_as_json)
    }
}
