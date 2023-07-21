use anyhow::{bail, Result};
use serde_json::{json, Value as JsonValue};
use std::time::SystemTime;

use reqwest::header::{HeaderValue, COOKIE};
use reqwest::Url;
use reqwest::{Client, Response, StatusCode};

use crate::error::{Result as UrbitResult, UrbitAPIError};
use rand::Rng;

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

    pub async fn open_channel(&mut self) -> Result<(Url, String, String)> {
        let mut rng = rand::thread_rng();
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
                "id": 1,
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

        self.channel_url.replace(channel_url);

        Ok((
            url_structured,
            ship_name.to_string(),
            session_auth.to_string(),
        ))
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

        println!(
            "ship: [start_listener] opening channel [{}, {}, {}]...",
            url, session_auth, json
        );

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
            if result.status() != 200 {
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
    pub async fn post(&mut self, payload: &JsonValue) -> Result<()> {
        let session_auth = self.session_auth.as_ref().unwrap().to_string();
        let channel_url = self.channel_url.as_ref().unwrap().to_string();
        let post_result: () = 'result: {
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
            ()
        };
        Ok(post_result)
    }
}
