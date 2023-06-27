use serde_json::{json, Value};
use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

use crate::types::{DispatcherResult, Path, WebSocketMessageHandler};

// The struct which holds the details for connecting to a given Urbit ship
#[derive(Debug, Clone)]
pub struct Dispatcher {
    subs: HashMap<Path, WebSocketMessageHandler>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            subs: HashMap::new(),
        }
    }

    pub fn add_sub(&mut self, path: Path, sub: WebSocketMessageHandler) {
        self.subs.insert(path, sub);
    }

    pub fn get_sub(&self, path: &str) -> Option<&WebSocketMessageHandler> {
        self.subs.get(path)
    }

    pub async fn dispatch(
        &mut self,
        sender: UnboundedSender<Message>,
        path: Path,
        message: Value,
    ) -> DispatcherResult {
        let sub = self.get_sub(path.as_str());
        if sub.is_none() {
            println!("no sub found for path '{}'. skipping...", path.as_str());
            return Ok(json!({}));
        }
        sub.unwrap().clone()(sender, message);
        // sub.unwrap().on_message(sender, message);
        return Ok(json!({}));
    }
}
