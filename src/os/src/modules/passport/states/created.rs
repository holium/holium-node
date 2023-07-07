use crate::modules::passport::{
    action::IdentityAction, effect::IdentityEffect, identity_state::GlobalIdentityState,
};
use std::any::Any;

use super::{CloneState, IdentityState};

#[derive(Clone)]
pub struct Created;

impl Created {
    pub fn new() -> Self {
        Created
    }
}

impl IdentityState for Created {
    fn perform_action(
        &self,
        action: IdentityAction,
        module: &mut GlobalIdentityState,
    ) -> Vec<IdentityEffect> {
        // Perform actions relevant to the Initializing state here
        // TODO setup the persistance prior to initializing
        vec![]
    }
}

impl CloneState for Created {
    fn clone_box(&self) -> Box<dyn IdentityState> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
