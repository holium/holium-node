use std::time::SystemTime;

use super::action::IdentityAction;

#[derive(Clone)]
pub struct GlobalIdentityState {
    pub identity: String,
    pub is_write_in_progress: bool,
    pub next_action: Option<IdentityAction>,
}

impl GlobalIdentityState {
    pub fn new() -> GlobalIdentityState {
        GlobalIdentityState {
            identity: "lomder-librun".to_string(),
            is_write_in_progress: false,
            next_action: None,
        }
    }

    pub fn apply_change(&mut self, change: &GlobalIdentityStateChange) {
        println!("applying change {:?}", change);
        match change {
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum GlobalIdentityStateChange {}
