use conduit::holon::OurPeer;

use self::{
    action::IdentityAction,
    effect::IdentityEffect,
    identity_state::GlobalIdentityState,
    states::{initializing::Initializing, IdentityState},
};

mod action;
mod effect;
mod identity_state;
mod states;

pub struct PassportModule {
    pub our_peer: Option<OurPeer>,
    current_state: Box<dyn IdentityState>,
    identity_state: GlobalIdentityState,
    // event_log: Vec<Event>,
}

impl PassportModule {
    pub fn new() -> Self {
        let current_state = Box::new(Initializing::new());
        return PassportModule {
            our_peer: None,
            current_state,
            identity_state: GlobalIdentityState::new(),
        };
    }

    pub fn initialize(&mut self, our_identity: OurPeer) {
        self.current_state = Box::new(Initializing::new());
        self.our_peer = Some(our_identity);
    }

    // transition functions
    fn transition(&mut self, action: IdentityAction) {
        let effects = self
            .current_state
            .perform_action(action, &mut self.identity_state);
        for effect in &effects {
            match effect {
                IdentityEffect::StateTransition { ref next_state } => {
                    self.current_state = next_state.clone();
                }
                _ => {}
            }
        }
    }
}
