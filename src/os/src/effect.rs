use crate::{global_state::GlobalStateChange, states::StateBox};
use std::fmt;

#[derive(Clone)]
pub enum Effect {
    GlobalStateChange(GlobalStateChange),
    StateTransition(StateBox),
}

impl fmt::Debug for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::GlobalStateChange(global_state_change) => {
                write!(f, "GlobalStateChange({:?})", global_state_change)
            }
            Effect::StateTransition(state) => write!(f, "StateTransition({:?})", state.describe()),
            // _ => {
            //     write!(f, "Unimplemented effect")
            // }
        }
    }
}
