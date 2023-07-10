use std::time::SystemTime;

use crate::modules::passport::contracts::IdentitySystem;

#[derive(Clone, Debug)]
pub enum Action {
    Preboot(PrebootArgs),
    Boot(BootArgs),
    Shutdown(),
    // Active actions
    // ReadData {
    //     path: String,
    //     data: Vec<u8>,
    // },
    // WriteData {
    //     path: String,
    //     data: Vec<u8>,
    // },
    // StartService {
    //     path: String,
    //     data: Vec<u8>,
    // },
    // StopService {
    //     path: String,
    //     data: Vec<u8>,
    // },

    // // Idle actions
    // CleanupResource {
    //     path: String,
    //     data: Vec<u8>,
    // },
}

#[derive(Clone, Debug)]
pub struct PrebootArgs {
    pub hid: String,
    pub boot_key: String,
    pub bind_address: String,
    pub identity_system: IdentitySystem,
}

#[derive(Clone, Debug)]
pub struct BootArgs {
    pub port: Option<i16>,
}

#[derive(Clone, Debug)]
pub struct GetUpdateArgs {
    pub current_version: String,
}

#[derive(Clone, Debug)]
pub struct GetPeerTableArgs {
    pub last_updated: SystemTime,
}

// describe action
impl Action {
    pub fn description(&self) -> String {
        match self {
            Action::Preboot(_) => String::from("preboot"),
            Action::Boot(_) => String::from("boot"),
            Action::Shutdown() => String::from("shutdown"),
        }
    }
}
