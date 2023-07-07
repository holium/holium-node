use crate::state::CloneState;
use crate::OS;
use std::any::Any;
use std::time::SystemTime;

use crate::action::Action;
use crate::effect::{Effect, Effects};
use crate::global_state::GlobalStateChange;
use crate::states::idle::Idle;

use super::super::state::State;

#[derive(Clone)]
pub struct Boot;

/// # Boot
/// Boot is the first state of the system. It is responsible for fetching identity data
/// from an eth node and starting the system services.
///
impl Boot {
    pub fn new() -> Boot {
        Boot
    }

    pub fn boot_with_identity(&self) {}

    pub fn boot(&self) {
        println!("Starting boot process...");
        // self.start_system_services();
    }
}

impl State for Boot {
    fn perform_action(&self, action: Action, os_state: &OS) -> Vec<Effect> {
        match action {
            Action::Boot { path, data: _ } => {
                self.boot();
                // Do whatever with state
                vec![
                    Effect {
                        path: path.clone(),
                        effect: Effects::GlobalStateChange(GlobalStateChange::SystemBooted(
                            SystemTime::now(),
                        )),
                        data: vec![],
                    },
                    Effect {
                        path: path.clone(),
                        effect: Effects::StateTransition(Box::new(Idle::new())),
                        data: vec![],
                    },
                ]
            }
            _ => vec![],
        }
    }
}

impl CloneState for Boot {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
