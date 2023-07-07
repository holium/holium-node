use std::time::SystemTime;

use crate::action::Action;

#[derive(Clone)]
pub struct GlobalState {
    pub identity: String,
    pub is_write_in_progress: bool,
    pub next_action: Option<Action>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            identity: "lomder-librun".to_string(),
            is_write_in_progress: false,
            next_action: None,
        }
    }

    pub fn apply_change(&mut self, change: &GlobalStateChange) {
        println!("applying change {:?}", change);
        match change {
            GlobalStateChange::IdentityPending(_) => {}
            GlobalStateChange::IdentityVerified(_) => {}
            GlobalStateChange::SystemBooted(_) => {}
            GlobalStateChange::SystemShutdown => {}
            GlobalStateChange::DataWritten => {}
            GlobalStateChange::DataDeleted => {}
            GlobalStateChange::ServiceStarted(_) => {}
            GlobalStateChange::ServiceStopped(_) => {}
            GlobalStateChange::ResourcesCleaned => {}
            GlobalStateChange::ResourcesAllocated => {}
            GlobalStateChange::WriteInProgress(value) => {
                self.is_write_in_progress = *value;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum GlobalStateChange {
    IdentityPending(String),
    IdentityVerified(String),
    SystemBooted(SystemTime),
    SystemShutdown,
    DataWritten,
    DataDeleted,
    ServiceStarted(String),
    ServiceStopped(String),
    ResourcesCleaned,
    ResourcesAllocated,
    WriteInProgress(bool),
}
