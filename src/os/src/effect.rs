use crate::{global_state::GlobalStateChange, state::State};
use std::fmt;

#[derive(Clone)]
pub enum Effects {
    DataRead,
    DataWritten,
    ServiceStarted,
    ServiceStopped,
    GlobalStateChange(GlobalStateChange),
    StateTransition(Box<dyn State>),
}

impl fmt::Debug for Effects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effects::GlobalStateChange(global_state_change) => {
                write!(f, "GlobalStateChange({:?})", global_state_change)
            }
            _ => {
                write!(f, "Unimplemented action")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub path: String,
    pub effect: Effects,
    pub data: Vec<u8>, // Output data
}
