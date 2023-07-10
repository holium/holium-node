use async_trait::async_trait;

use crate::action::Action;
use crate::states::{CloneState, State};
use crate::OS;

use super::{ActionResult, StateBox};

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

#[async_trait(?Send)]
impl State for Idle {
    fn describe(&self) -> String {
        String::from("idle")
    }
    async fn perform_action(&self, action: Action, _os_state: &OS) -> ActionResult {
        match action {
            // Action::CleanupResource { path, data: _ } => {
            //     self.cleanup_resources();
            //     vec![Effect::GlobalStateChange(
            //         GlobalStateChange::ResourcesCleaned,
            //     )]
            // }
            _ => ActionResult::Ok(vec![]),
        }
    }
}

impl CloneState for Idle {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}
