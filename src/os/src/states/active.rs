use crate::state::CloneState;
use crate::OS;
use std::any::Any;

use super::super::state::State;
use crate::action::Action;
use crate::effect::{Effect, Effects};

#[derive(Clone)]
pub struct Active;

impl Active {
    pub fn new() -> Active {
        Active
    }
}

impl State for Active {
    fn perform_action(&self, action: Action, os_state: &OS) -> Vec<Effect> {
        match action {
            // Action::ReadData { path, data: _ } => {
            //     println!("Reading data from system resources");
            //     vec![Effect {
            //         path: path,
            //         effect: Effects::DataRead,
            //         data: vec![],
            //     }]
            // }
            Action::WriteData { path, data: _ } => {
                vec![Effect {
                    path: path,
                    effect: Effects::DataWritten,
                    data: vec![],
                }]
            }
            Action::StartService { path, data: _ } => {
                println!("Starting a service");
                vec![Effect {
                    path: path,
                    effect: Effects::ServiceStarted,
                    data: vec![],
                }]
            }
            Action::StopService { path, data: _ } => {
                println!("Stopping a service");
                vec![Effect {
                    path: path,
                    effect: Effects::ServiceStopped,
                    data: vec![],
                }]
            }
            _ => {
                // For any other action, we do nothing.
                vec![]
            }
        }
    }
}

impl CloneState for Active {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn start_network_runtime() {
    // Implementation for starting the network runtime
    println!("Starting network runtime...");
    // Code to start listening for network events
    // ...
}
