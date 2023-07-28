use anyhow::{bail, Result};
use serde_json::{json, Value as JsonValue};
use std::time::SystemTime;

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::Url;
use reqwest::{Client, Response, StatusCode};

use eventsource_threaded::{EventSource, ReceiverSource};

use crate::error::{Result as UrbitResult, UrbitAPIError};
use rand::Rng;

use trace::{trace_err_ln, trace_green_ln, trace_info_ln, trace_json_ln, trace_warn_ln};

pub static SUBSCRIPTION_MSG_ID: u64 = u64::MAX - 1;
// static NEXT_MESSAGE_ID: AtomicU64 = AtomicU64::new(u64::MAX - 1000);

#[derive(Debug, Clone)]
pub struct Ship {
    /// The URL of the ship given as `http://ip:port` such as
    /// `http://0.0.0.0:8080`.
    pub url: String,
    // ship code
    ship_code: String,
    // channel url is generated with the open_channel function is called
    pub channel_url: Option<String>,
    /// The session auth string header value
    pub session_auth: Option<String>,
    /// The ship name (without a leading ~)
    pub ship_name: Option<String>,
    /// The Reqwest `Client` to be reused for making requests
    req_client: Client,
}

impl Ship {
    pub async fn new(url: &str, ship_code: &str) -> Result<Ship> {
        let mut result = Ship {
            url: url.to_string(),
            ship_code: ship_code.to_string(),
            channel_url: None,
            session_auth: None,
            ship_name: None,
            req_client: Client::new(),
        };
        match result.login().await {
            Ok(_) => Ok(result),
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn login(&mut self) -> Result<(String, String)> {
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

        let ship_name = ship_name;
        let session_auth = session_auth;

        self.ship_name.replace(ship_name.to_string());
        self.session_auth.replace(session_auth.to_string());

        Ok((ship_name.to_string(), session_auth.to_string()))
    }

    // 1) poke the ship to open the channel @ url
    // 2) connect the event source to the channel url
    // 3) read in the first event from the EventSource and
    //    validate that it matches the sub's message id and
    //    with a payload containing response='poke' and ok='ok' fields
    // then, and only then, do you return success from this method
    pub async fn open_channel(&mut self) -> Result<ReceiverSource> {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_entropy();
        // Defining the uid as UNIX time, or random if error
        let uid = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis(),
            Err(_) => rng.gen(),
        }
        .to_string();

        // Channel url
        let channel_url = format!("{}/~/channel/{}", &self.url, uid);

        // Create the receiver
        let url_structured = {
            let url_structured = Url::parse(&channel_url); //.map_err(|_| UrbitAPIError::FailedToCreateNewChannel)?;

            if url_structured.is_err() {
                bail!("chat: [listen] error parsing channel url {}", channel_url)
            }

            url_structured.unwrap()
        };

        let ship_name = self.ship_name.as_ref().unwrap().to_string();
        let session_auth = self.session_auth.as_ref().unwrap().to_string();

        // Opening channel request json
        let body = json!([{
                "id": SUBSCRIPTION_MSG_ID,
                "action": "poke",
                "ship": ship_name,
                "app": "hood",
                "mark": "helm-hi",
                "json": "Opening channel",
        }]);

        // Make the put request to create the channel.
        let resp = self
            .send_put_request(&channel_url, session_auth.as_str(), &body)
            .await;

        if resp.is_err() {
            bail!("ship: [start_listener] failed to open channel. put request failed.");
        }

        let resp = resp.unwrap();

        if resp.status().as_u16() != 204 {
            bail!(
                "ship: [start_listener] failed to open channel. {}",
                resp.status().as_u16()
            );
        }

        // Create cookie header with the ship session auth val
        let mut headers = HeaderMap::new();
        headers.append(COOKIE, HeaderValue::from_str(&session_auth)?);

        // now open the EventSource and retrieve/ack the poke event from above.
        //  only after ALL of this succeeds should we return the receiver and start polling
        let receiver = EventSource::new(url_structured, headers);
        trace_info_ln!("waiting for open channel confirmation event...");

        let msg = receiver.recv();

        if msg.is_err() {
            bail!("api: [open_channel] event receive error. msg => {:?}", msg);
        }

        let msg = msg.unwrap();

        if msg.is_err() {
            bail!("api: [open_channel] event receive error. msg =>{:?}", msg);
        }

        // the deserialized Event from SSE
        let event = msg.unwrap();

        trace_green_ln!("api: [open_channel] received event:");

        let data = serde_json::from_str::<JsonValue>(&event.data);

        if data.is_err() {
            bail!(
                "api: [open_channel] error deserializing confirmation event => {:?}",
                event.data
            );
        }

        let data = data.unwrap();

        if !data.is_object() {
            bail!(
                "api: [open_channel] invalid event stream message. must be json object {}",
                event.data
            );
        }

        trace_json_ln!(&data);

        let id = {
            let id = data.get("id");
            if id.is_none() {
                bail!(
                    "api: [open_channel] invalid event stream message. must be json object {}",
                    event.data
                )
            }
            let id = id.unwrap().as_u64();
            if id.is_none() {
                bail!(
                    "api: [open_channel] SSE event id not a valid number {}",
                    event.data
                )
            }
            id.unwrap()
        };

        let ok = {
            let ok = data.get("ok");
            if ok.is_none() {
                bail!(
                    "api: [open_channel] unexpected SSE event. missing ok field. {}",
                    event.data
                )
            }
            ok.unwrap()
        };

        let response = {
            let response = data.get("response");
            if response.is_none() {
                bail!(
                    "api: [open_channel] unexpected SSE event. missing response field. {}",
                    event.data
                )
            }
            response.unwrap()
        };

        if !(data.is_object()
            && id == super::api::SUBSCRIPTION_MSG_ID
            && ok == "ok"
            && response == "poke")
        {
            bail!(
                "api: [open_channel] failed to valid SSE handshake {}",
                event.data
            )
        }

        self.channel_url.replace(channel_url);

        Ok(receiver)
        // Ok((
        //     url_structured,
        //     ship_name.to_string(),
        //     session_auth.to_string(),
        // ))
    }

    // Send a put request using the `ShipInterface`
    pub async fn send_put_request(
        &self,
        url: &str,
        session_auth: &str,
        body: &JsonValue,
    ) -> Result<Response> {
        // let json = body.to_string();
        let json = serde_json::to_string(body)?;

        trace_info_ln!("opening channel:");
        trace_json_ln!(&json!({
          "url": url,
          "session_auth": session_auth,
          "json": &body
        })); //  [{}, {}, {}]...", url, session_auth, json);
        trace_info_ln!("opening channel @ {}...", url);

        let req = self
            .req_client
            .put(url)
            .header(COOKIE, HeaderValue::from_str(session_auth)?)
            .header("Content-Type", "application/json")
            .body(json);

        let res = req.send().await;

        if res.is_err() {
            bail!("ship: [send_put_request] request failed");
        }

        let res = res.unwrap();

        if res.status().as_u16() != 204 {
            bail!(
                "ship: [send_put_request] request failed. {}",
                res.status().as_u16()
            );
        }

        Ok(res)
    }

    /// Sends a scry to the ship
    pub async fn scry(&mut self, app: &str, path: &str, mark: &str) -> UrbitResult<JsonValue> {
        let scry_url = format!("{}/~/scry/{}{}.{}", self.url, app, path, mark);
        let session_auth = self.session_auth.as_ref().unwrap().to_string();
        let ship_response_as_json: JsonValue = 'response_json: {
            let resp = self
                .req_client
                .get(&scry_url)
                .header(COOKIE, session_auth.to_string())
                .header("Content-Type", "application/json");
            let result = resp.send().await?;
            if result.status().as_u16() == StatusCode::FORBIDDEN {
                trace_warn_ln!("ship: [scry] session expired. logging in...");
                let result = self.login().await;
                if result.is_err() {
                    trace_err_ln!("login failed");
                    return Err(UrbitAPIError::FailedToLogin);
                }
                let resp = self
                    .req_client
                    .get(&scry_url)
                    .header(COOKIE, session_auth)
                    .header("Content-Type", "application/json");
                let result = resp.send().await?;
                if result.status().as_u16() != 200 {
                    trace_err_ln!("retry failed. error {}", result.status().as_u16());
                    return Err(UrbitAPIError::StatusCode(result.status().as_u16()));
                }
                break 'response_json result.json().await.unwrap();
            }
            if result.status() != 200 {
                trace_err_ln!("failed to post payload");
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
    pub async fn post(&mut self, payload: &JsonValue) -> Result<()> {
        let session_auth = self.session_auth.as_ref().unwrap().to_string();
        let channel_url = self.channel_url.as_ref().unwrap().to_string();
        let post_result: () = 'result: {
            trace_info_ln!(
                "posting message to '{}'...",
                self.channel_url.as_ref().unwrap().to_string()
            );
            let res = self
                .req_client
                .post(channel_url.to_string())
                .header(COOKIE, session_auth.to_string())
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await?;
            // if it's a 403, this indicates the auth header is invalid or expired
            //  try to fetch another token and retry
            if res.status() == StatusCode::FORBIDDEN {
                trace_warn_ln!(
                    "403. retrying. posting message to '{}'...",
                    self.channel_url.as_ref().unwrap().to_string()
                );
                let result = self.login().await;
                if result.is_err() {
                    bail!("ship: [post] login failed")
                }
                let res = self
                    .req_client
                    .post(channel_url.to_string())
                    .header(COOKIE, session_auth.to_string())
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;
                if res.status().as_u16() != 204 {
                    bail!("ship: [post] retry failed. {}", res.status().as_u16());
                }
                break 'result ();
            }
            if res.status().as_u16() != 204 {
                bail!(
                    "ship: [post] failed to post payload. {}",
                    res.status().as_u16()
                )
            }
            trace_green_ln!("ship: [post] success {}", payload.to_string());
            ()
        };
        Ok(post_result)
    }
}
