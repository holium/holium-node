use super::{CloneState, IdentityState};
use std::any::Any;
use std::vec;

use crate::{
    action::Action,
    modules::passport::{
        action::IdentityAction, effect::IdentityEffect, identity_state::GlobalIdentityState,
    },
};

#[derive(Clone)]
pub struct Ready;

impl Ready {
    pub fn new() -> Self {
        Ready
    }
}

impl IdentityState for Ready {
    fn perform_action(
        &self,
        action: Action,
        module: &mut GlobalIdentityState,
    ) -> Vec<IdentityEffect> {
        // Perform actions relevant to the Initializing state here
        vec![]
    }
}

impl CloneState for Ready {
    fn clone_box(&self) -> Box<dyn IdentityState> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
