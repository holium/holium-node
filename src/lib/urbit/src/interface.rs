use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Result, UrbitAPIError};
use serde_json::Value;

use reqwest::header::{HeaderValue, COOKIE};
use reqwest::{Client, Response};

use crate::Channel;

// The struct which holds the details for connecting to a given Urbit ship
#[derive(Debug, Clone)]
pub struct ShipInterface {
    /// The URL of the ship given as `http://ip:port` such as
    /// `http://0.0.0.0:8080`.
    pub url: String,
    /// The session auth string header value
    pub session_auth: Option<HeaderValue>,
    /// The ship name (without a leading ~)
    pub ship_name: Option<String>,
    /// The Reqwest `Client` to be reused for making requests
    req_client: Client,
    ship_code: String,
}

#[derive(Debug, Clone)]
pub struct SafeShipInterface {
    api: Arc<Mutex<ShipInterface>>,
}

/*
  Provide a thread-safe wrapper around the underlying ShipInterface
*/
impl SafeShipInterface {
    pub async fn new(url: &str, code: &str) -> Result<SafeShipInterface> {
        let api = ShipInterface::new(url, code).await?;
        Ok(SafeShipInterface {
            api: Arc::new(Mutex::new(api)),
        })
    }

    pub async fn refresh(&self) -> Result<()> {
        let mut api = self.api.lock().await;
        api.refresh().await
    }

    pub async fn scry(&self, app: &str, path: &str, mark: &str) -> Result<Value> {
        let api = self.api.lock().await;
        api.scry(app, path, mark).await
    }
}

impl ShipInterface {
    /// Logs into the given ship and creates a new `ShipInterface`.
    /// `ship_url` should be `http://ip:port` of the given ship. Example:
    /// `http://0.0.0.0:8080`. `ship_code` is the code acquire from your ship
    /// by typing `+code` in dojo.
    pub async fn new(ship_url: &str, ship_code: &str) -> Result<ShipInterface> {
        let mut result = ShipInterface {
            url: ship_url.to_string(),
            session_auth: None, // HeaderValue::from_str("").unwrap(),
            ship_name: None,    //
            req_client: Client::new(),
            ship_code: ship_code.to_string(),
        };
        match result.refresh().await {
            Ok(_) => Ok(result),
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let login_url = format!("{}/~/login", self.url);
        let resp = self
            .req_client
            .post(&login_url)
            .body("password=".to_string() + &self.ship_code)
            .send()
            .await?;

        // Check for status code
        if resp.status().as_u16() != 204 {
            return Err(UrbitAPIError::FailedToLogin);
        }

        // Acquire the session auth header value
        let session_auth = resp
            .headers()
            .get("set-cookie")
            .ok_or(UrbitAPIError::FailedToLogin)?;

        // Convert sessions auth to a string
        let auth_string = session_auth
            .to_str()
            .map_err(|_| UrbitAPIError::FailedToLogin)?;

        self.session_auth = Some(session_auth.clone());

        // Trim the auth string to acquire the ship name
        let end_pos = auth_string.find('=').ok_or(UrbitAPIError::FailedToLogin)?;
        let ship_name = &auth_string[9..end_pos];

        self.ship_name = Some(ship_name.to_string());

        Ok(())
    }

    /// Returns the ship name with a leading `~` (By default ship_name does not have one)
    pub fn ship_name_with_sig(&self) -> String {
        format!("~{}", self.ship_name.as_ref().unwrap().to_string())
    }

    /// Create a `Channel` using this `ShipInterface`
    pub async fn create_channel(&self) -> Result<Channel> {
        Channel::new(self.clone()).await
    }

    // Send a put request using the `ShipInterface`
    pub async fn send_put_request(&self, url: &str, body: &Value) -> Result<Response> {
        let json = body.to_string();
        let resp = self
            .req_client
            .put(url)
            .header(COOKIE, self.session_auth.clone().unwrap())
            .header("Content-Type", "application/json")
            .body(json);

        Ok(resp.send().await?)
    }

    /// Sends a scry to the ship
    pub async fn scry(&self, app: &str, path: &str, mark: &str) -> Result<Value> {
        let scry_url = format!("{}/~/scry/{}{}.{}", self.url, app, path, mark);
        let resp = self
            .req_client
            .get(&scry_url)
            .header(COOKIE, self.session_auth.clone().unwrap())
            .header("Content-Type", "application/json");
        let result = resp.send().await?;
        if result.status().as_u16() == 200 {
            Ok(result.json::<Value>().await?)
        } else {
            match result.status().as_u16() {
                403 => Err(UrbitAPIError::Forbidden),
                500 => Err(UrbitAPIError::ServerError),
                _ => Err(UrbitAPIError::Other(format!(
                    "unexpected error: {}",
                    result.status().as_u16()
                ))),
            }
        }
    }

    /// Run a thread via spider
    pub async fn spider(
        &self,
        input_mark: &str,
        output_mark: &str,
        thread_name: &str,
        body: &Value,
    ) -> Result<Response> {
        let json = body.to_string();
        let spider_url = format!(
            "{}/spider/{}/{}/{}.json",
            self.url, input_mark, thread_name, output_mark
        );

        let resp = self
            .req_client
            .post(&spider_url)
            .header(COOKIE, self.session_auth.clone().unwrap())
            .header("Content-Type", "application/json")
            .body(json);

        Ok(resp.send().await?)
    }
}

// impl Default for ShipInterface {
//     async fn default() -> Self {
//         ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup")
//             .await
//             .unwrap()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::subscription::Subscription;
//     use json::object;
//     #[test]
//     // Verify that we can login to a local `~zod` dev ship.
//     fn can_login() {
//         let ship_interface =
//             ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
//     }

//     #[test]
//     // Verify that we can create a channel
//     fn can_create_channel() {
//         let ship_interface =
//             ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
//         let channel = ship_interface.create_channel().unwrap();
//         channel.delete_channel();
//     }

//     #[test]
//     // Verify that we can create a channel
//     fn can_subscribe() {
//         let ship_interface =
//             ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
//         let mut channel = ship_interface.create_channel().unwrap();
//         channel
//             .create_new_subscription("chat-view", "/primary")
//             .unwrap();

//         channel.find_subscription("chat-view", "/primary");
//         channel.unsubscribe("chat-view", "/primary");
//         channel.delete_channel();
//     }

//     #[test]
//     // Verify that we can make a poke
//     fn can_poke() {
//         let ship_interface =
//             ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
//         let mut channel = ship_interface.create_channel().unwrap();
//         let poke_res = channel
//             .poke("hood", "helm-hi", &"A poke has been made".into())
//             .unwrap();
//         assert!(poke_res.status().as_u16() == 204);
//         channel.delete_channel();
//     }

//     #[test]
//     // Verify we can scry
//     fn can_scry() {
//         let mut ship_interface =
//             ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
//         let scry_res = ship_interface.scry("graph-store", "/keys", "json").unwrap();

//         assert!(scry_res.status().as_u16() == 200);
//     }

//     #[test]
//     // Verify we can run threads
//     fn can_spider() {
//         let mut ship_interface =
//             ShipInterface::new("http://0.0.0.0:8080", "lidlut-tabwed-pillex-ridrup").unwrap();
//         let create_req = object! {
//             "create": {
//                 "resource": {
//                     "ship": "~zod",
//                     "name": "test",
//                 },
//                 "title": "Testing creation",
//                 "description": "test",
//                 "associated": {
//                     "policy": {
//                         "invite": {
//                             "pending": []
//                         }
//                     }
//                 },
//                 "module": "chat",
//                 "mark": "graph-validator-chat"
//             }
//         };

//         let spider_res = ship_interface
//             .spider("graph-view-action", "json", "graph-create", &create_req)
//             .unwrap();

//         assert!(spider_res.status().as_u16() == 200);
//     }
// }
