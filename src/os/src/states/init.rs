use crate::action::Action;
use crate::effect::{Effect, Effects};
use crate::state::CloneState;
use crate::OS;
use std::any::Any;
use std::vec;

use super::super::state::State;

use super::boot::Boot;

#[derive(Clone)]
pub struct Init;

impl Init {
    pub fn new() -> Init {
        println!("state: Init");
        Init
    }

    pub fn start_preboot(&self) {
        println!("Starting preboot process");
        // TODO prechecks
    }
}

impl State for Init {
    fn perform_action(&self, action: Action, os_state: &OS) -> Vec<crate::effect::Effect> {
        match action {
            Action::Preboot { path, data: _ } => {
                // 1. Get command line arguments
                // 2. Fetch identity data from eth node
                // 3. Fetch any updates from service node
                // 4. Create any directories and files
                self.start_preboot();

                vec![Effect {
                    path: path.clone(),
                    effect: Effects::StateTransition(Box::new(Boot::new())),
                    data: vec![],
                }]
            }
            _ => vec![],
        }
    }
}

impl CloneState for Init {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
