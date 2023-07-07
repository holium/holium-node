use super::super::state::State;
use crate::action::Action;
use crate::effect::{Effect, Effects};
use crate::global_state::GlobalStateChange;
use crate::state::CloneState;
use crate::OS;
use std::any::Any;

#[derive(Clone)]
pub struct Shutdown;

impl Shutdown {
    pub fn new() -> Self {
        Shutdown
    }

    pub fn shutdown_cleanly(&self) {
        println!("Shutdown sequence started");
        // Simulate writing any pending effects to disk
        println!("Writing pending effects to disk");
        // Simulate safe shutdown of system services
        println!("Safely shutting down system services");
        println!("Shutdown sequence completed");
    }
}

impl State for Shutdown {
    fn perform_action(&self, action: Action, os_state: &OS) -> Vec<Effect> {
        match action {
            Action::Shutdown { path, data: _ } => {
                self.shutdown_cleanly();
                vec![Effect {
                    path,
                    effect: Effects::GlobalStateChange(GlobalStateChange::SystemShutdown),
                    data: vec![],
                }]
            }
            _ => vec![],
        }
    }
    // fn as_any(&self) -> &dyn std::any::Any {
    //     self
    // }
}

impl CloneState for Shutdown {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
