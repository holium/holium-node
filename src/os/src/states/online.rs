use async_trait::async_trait;

use crate::states::{CloneState, State};
use crate::OS;

use crate::action::Action;

use super::{ActionResult, StateBox};

#[derive(Clone)]
pub struct Online;

impl Online {
    pub fn new() -> Online {
        Online
    }
}

#[async_trait(?Send)]
impl State for Online {
    fn describe(&self) -> String {
        String::from("online")
    }
    async fn perform_action(&self, action: Action, _os_state: &OS) -> ActionResult {
        match action {
            // Action::ReadData { path, data: _ } => {
            //     println!("Reading data from system resources");
            //     vec![Effect {
            //         path: path,
            //         effect: Effects::DataRead,
            //         data: vec![],
            //     }]
            // }
            // Action::WriteData { path, data: _ } => {
            //     vec![Effect::DataWritten {
            //         path: path,
            //         data: vec![],
            //     }]
            // }
            // Action::StartService { path, data: _ } => {
            //     println!("Starting a service");
            //     vec![Effect::ServiceStarted {
            //         path: path,
            //         data: vec![],
            //     }]
            // }
            // Action::StopService { path, data: _ } => {
            //     println!("Stopping a service");
            //     vec![Effect::ServiceStopped {
            //         path: path,
            //         data: vec![],
            //     }]
            _ => {
                // For any other action, we do nothing.
                ActionResult::Ok(vec![])
            }
        }
    }
}

impl CloneState for Online {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}

// fn start_network_runtime() {
//     // Implementation for starting the network runtime
//     println!("Starting network runtime...");
//     // Code to start listening for network events
//     // ...
// }
