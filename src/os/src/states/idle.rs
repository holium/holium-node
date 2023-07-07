use crate::state::CloneState;
use crate::OS;
use std::any::Any;

use super::super::state::State;
use crate::action::Action;
use crate::effect::{Effect, Effects};
use crate::global_state::GlobalStateChange;

#[derive(Clone)]
pub struct Idle;

impl Idle {
    pub fn new() -> Self {
        Idle
    }

    pub fn cleanup_resources(&self) {
        println!("Cleanup of system resources started");
        // Simulate resource cleanup
    }
}

impl State for Idle {
    fn perform_action(&self, action: Action, os_state: &OS) -> Vec<Effect> {
        match action {
            Action::CleanupResource { path, data: _ } => {
                self.cleanup_resources();
                vec![Effect {
                    path,
                    effect: Effects::GlobalStateChange(GlobalStateChange::ResourcesCleaned),
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

// impl fmt::Debug for Idle {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Idle")
//     }
// }

impl CloneState for Idle {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
