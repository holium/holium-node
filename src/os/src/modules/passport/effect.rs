use super::{identity_state::GlobalIdentityStateChange, states::IdentityState};

#[derive(Clone, Debug)]
pub enum IdentityEffect {
    GlobalStateChange { change: GlobalIdentityStateChange },
    StateTransition { next_state: Box<dyn IdentityState> },
}
