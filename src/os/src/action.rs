use std::time::SystemTime;

#[derive(Clone, Debug)]
pub enum Action {
    // Init actions
    Preboot {
        path: String,
        data: PrebootArgs,
    },
    GetUpdates {
        path: String,
        data: GetUpdateArgs,
    },
    GetPeerTable {
        path: String,
        data: GetPeerTableArgs,
    },

    // Boot actions
    Boot {
        path: String,
        data: Vec<u8>,
    },

    // Shutdown actions
    Shutdown {
        path: String,
        data: Vec<u8>,
    },

    // Active actions
    ReadData {
        path: String,
        data: Vec<u8>,
    },
    WriteData {
        path: String,
        data: Vec<u8>,
    },
    StartService {
        path: String,
        data: Vec<u8>,
    },
    StopService {
        path: String,
        data: Vec<u8>,
    },

    // Idle actions
    CleanupResource {
        path: String,
        data: Vec<u8>,
    },
}

#[derive(Clone, Debug)]
pub struct PrebootArgs {
    pub hid: String,
    pub boot_key: String,
}

#[derive(Clone, Debug)]
pub struct GetUpdateArgs {
    pub current_version: String,
}

#[derive(Clone, Debug)]
pub struct GetPeerTableArgs {
    pub last_updated: SystemTime,
}
