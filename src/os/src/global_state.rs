use crate::{
    action::Action,
    holon::OurPeer,
    modules::passport::{contracts::IdentitySystem, passport_state::PassportState},
};

#[derive(Clone)]
pub struct ModuleStates {
    pub passport: PassportState,
}

#[derive(Clone)]
pub struct GlobalState {
    pub is_write_in_progress: bool,
    pub modules: ModuleStates,
    pub next_action: Option<Action>,
}

impl GlobalState {
    pub fn genesis() -> GlobalState {
        GlobalState {
            is_write_in_progress: false,
            modules: ModuleStates {
                passport: PassportState::new(),
            },
            next_action: None,
        }
    }

    pub fn apply_change(&mut self, change: &GlobalStateChange) {
        println!("state-update:{}", change.to_string());
        match change {
            GlobalStateChange::PassportCreated(PassportSetData {
                identity,
                identity_system,
            }) => {
                self.modules.passport.identity = Some(identity.clone());
                self.modules.passport.identity_system = Some(identity_system.clone());
            }
            GlobalStateChange::Shutdown => {}
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
    PassportCreated(PassportSetData),
    Shutdown,
    DataWritten,
    DataDeleted,
    ServiceStarted(String),
    ServiceStopped(String),
    ResourcesCleaned,
    ResourcesAllocated,
    WriteInProgress(bool),
}

// describe the state change in a human readable way
impl std::fmt::Display for GlobalStateChange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GlobalStateChange::PassportCreated(_) => write!(f, "passport_created"),
            GlobalStateChange::Shutdown => write!(f, "holon_shutdown"),
            GlobalStateChange::DataWritten => write!(f, "data_written"),
            GlobalStateChange::DataDeleted => write!(f, "data_deleted"),
            GlobalStateChange::ServiceStarted(_) => write!(f, "service_started"),
            GlobalStateChange::ServiceStopped(_) => write!(f, "service_stopped"),
            GlobalStateChange::ResourcesCleaned => write!(f, "resources_cleaned"),
            GlobalStateChange::ResourcesAllocated => write!(f, "resources_allocated"),
            GlobalStateChange::WriteInProgress(_) => write!(f, "write_in_progress"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PassportSetData {
    pub identity: OurPeer,
    pub identity_system: IdentitySystem,
}
