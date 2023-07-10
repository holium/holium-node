use async_trait::async_trait;

use crate::action::Action;
use crate::effect::Effect;
use crate::global_state::GlobalStateChange;
use crate::states::{CloneState, State};
use crate::OS;

use super::{ActionResult, StateBox};

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

#[async_trait(?Send)]
impl State for Shutdown {
    fn describe(&self) -> String {
        String::from("shutdown")
    }
    async fn perform_action(&self, action: Action, _os_state: &OS) -> ActionResult {
        match action {
            Action::Shutdown() => {
                self.shutdown_cleanly();
                ActionResult::Ok(vec![Effect::GlobalStateChange(GlobalStateChange::Shutdown)])
            }
            _ => ActionResult::Ok(vec![]),
        }
    }
}

impl CloneState for Shutdown {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}
