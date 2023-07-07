pub mod created;
pub mod initializing;
pub mod ready;

use std::{any::Any, fmt};

use super::{action::IdentityAction, effect::IdentityEffect, identity_state::GlobalIdentityState};

pub trait IdentityState: CloneState {
    fn perform_action(
        &self,
        action: IdentityAction,
        global_state: &mut GlobalIdentityState,
    ) -> Vec<IdentityEffect>;
}

pub trait CloneState {
    fn clone_box(&self) -> Box<dyn IdentityState>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn IdentityState> {
    fn clone(&self) -> Box<dyn IdentityState> {
        self.clone_box()
    }
}

impl<T: 'static + IdentityState + fmt::Debug + Clone> CloneState for T {
    fn clone_box(&self) -> Box<dyn IdentityState> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Debug for Box<dyn IdentityState> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Attempt to downcast to the concrete state type
        if let Some(state) = self.as_any().downcast_ref::<Self>() {
            write!(f, "{:?}", state)
        } else {
            write!(f, "<unknown>")
        }
    }
}
