use anyhow::{bail, Result};
use serde_json::{json, Value as JsonValue};
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use crate::ShipInterface;
use reqwest::header::{HeaderValue, COOKIE};
use reqwest::{Client, Response, StatusCode};

#[derive(Debug, Clone)]
pub struct Ship {
    /// The URL of the ship given as `http://ip:port` such as
    /// `http://0.0.0.0:8080`.
    pub url: String,
    // ship code
    ship_code: String,
    /// The session auth string header value
    pub session_auth: Option<String>,
    /// The ship name (without a leading ~)
    pub ship_name: Option<String>,
    /// The Reqwest `Client` to be reused for making requests
    req_client: Client,
}

impl Ship {
    pub fn new(url: &str, ship_code: &str) -> Ship {
        Ship {
            url: url.to_string(),
            ship_code: ship_code.to_string(),
            ship_name: None,
            session_auth: None,
            req_client: reqwest::Client::new(),
        }
    }

    pub async fn login(&mut self) -> Result<()> {
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

        self.ship_name = Some(ship_name.to_string());
        // Convert sessions auth to a string
        self.session_auth = Some(session_auth.to_string());
        Ok(())
    }

    // use this method to forward actions posted over web socket connection
    //   originating from connected devices
    // this method will attempt to refresh the urbit auth cookie if the
    //   request fails with a 403 (forbidden).
    pub async fn post(&mut self, payload: &JsonValue) -> Result<JsonValue> {
        if self.ship_name.is_none() || self.session_auth.is_none() {
            bail!("ship: [post] must call login");
        }
        let ship_response_as_json: JsonValue = 'response_json: {
            let res = self
                .req_client
                .post(&self.url)
                .header(COOKIE, self.session_auth.clone().unwrap())
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
                    .header(COOKIE, self.session_auth.clone().unwrap())
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;
                // now if we've failed, simply bail without more attempts
                if res.status() != 204 {
                    bail!("ship: [post] failed to post payload")
                }
                break 'response_json res.json().await.unwrap(); //serde_json::from_str(&res.text().await?);
            }
            if res.status() != 204 {
                bail!("ship: [post] failed to post payload")
            }
            res.json().await.unwrap()
        };
        Ok(ship_response_as_json)
    }

    /// Sends a scry to the ship
    pub async fn scry_to_str(&self, app: &str, path: &str, mark: &str) -> Result<String> {
        let scry_url = format!("{}/~/scry/{}{}.{}", self.url, app, path, mark);
        let resp = self
            .req_client
            .get(&scry_url)
            .header(COOKIE, self.session_auth.clone().unwrap())
            .header("Content-Type", "application/json");
        let result = resp.send().await?;
        if result.status().as_u16() != 200 {
            bail!("ship: [scry_to_str] error {}", result.status().as_u16());
        }
        Ok(result.text().await?)
    }

    // Send a put request using the `ShipInterface`
    pub async fn send_put_request(&self, url: &str, body: &JsonValue) -> Result<Response> {
        let json = body.to_string();
        let resp = self
            .req_client
            .put(url)
            .header(COOKIE, self.session_auth.clone().unwrap())
            .header("Content-Type", "application/json")
            .body(json);

        Ok(resp.send().await?)
    }
}
