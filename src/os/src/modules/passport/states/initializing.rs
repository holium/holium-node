use std::any::Any;

use crate::modules::passport::{
    action::IdentityAction, effect::IdentityEffect, identity_state::GlobalIdentityState,
};

use super::{CloneState, IdentityState};

#[derive(Clone)]
pub struct Initializing;

impl Initializing {
    pub fn new() -> Self {
        Initializing
    }
}

impl IdentityState for Initializing {
    fn perform_action(
        &self,
        action: IdentityAction,
        module: &mut GlobalIdentityState,
    ) -> Vec<IdentityEffect> {
        // Perform actions relevant to the Initializing state here
        vec![]
    }
}

impl CloneState for Initializing {
    fn clone_box(&self) -> Box<dyn IdentityState> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
