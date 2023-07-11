use std::format;

use anyhow::{bail, Result};

use reqwest::header::COOKIE;
use reqwest::Client;

pub struct Ship {
    pub ship_name: Option<String>,
    pub url: String,
    pub code: String,
    pub auth_token: Option<String>,
    req: Option<Client>,
}

impl Ship {
    pub fn new(url: &str, code: &str) -> Self {
        Self {
            ship_name: None,
            url: url.to_string(),
            code: code.to_string(),
            auth_token: None,
            req: None,
        }
    }
    pub async fn login(&mut self) -> Result<()> {
        if self.req.is_none() {
            self.req = Some(Client::new());
        }
        let login_url = format!("{}/~/login", self.url);
        let resp = self
            .req
            .as_ref()
            .unwrap()
            .post(&login_url)
            .body("password=".to_string() + &self.code)
            .send()
            .await?;
        // Check for status code
        if resp.status().as_u16() != 204 {
            bail!("ship: login failed");
        }
        if !resp.headers().contains_key("set-cookie") {
            bail!("ship: login failed. cookie not found");
        }
        let auth_token = resp.headers().get("set-cookie").unwrap().to_str();
        if auth_token.is_err() {
            bail!("ship: login failed. cookie failure");
        }
        self.auth_token = Some(auth_token.unwrap().to_string());
        // Trim the auth string to acquire the ship name
        let end_pos = self.auth_token.as_ref().unwrap().find('=');
        if end_pos.is_none() {
            bail!("ship: login failed. invalid auth token");
        }
        let ship_name = &self.auth_token.as_ref().unwrap()[9..end_pos.unwrap()];
        self.ship_name = Some(ship_name.to_string());
        Ok(())
    }

    /// Sends a scry to the ship
    pub async fn scry(&self, app: &str, path: &str, mark: &str) -> Result<String> {
        let scry_url = format!("{}/~/scry/{}{}.{}", self.url, app, path, mark);
        let resp = self
            .req
            .as_ref()
            .unwrap()
            .get(scry_url)
            .header(COOKIE, self.auth_token.clone().unwrap())
            .header("Content-Type", "application/json");
        let result = resp.send().await?;
        if result.status().as_u16() != 200 {
            bail!("ship: scry failed {}", result.status().as_u16());
        }
        Ok(result.text().await?)
    }
}
