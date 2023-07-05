use super::ForeignPeerState;

pub trait StateChangeListener {
    fn on_state_change(&self, old_state: &ForeignPeerState, new_state: &ForeignPeerState);
}

pub struct PrintStateChangeListener;

impl StateChangeListener for PrintStateChangeListener {
    fn on_state_change(&self, old_state: &ForeignPeerState, new_state: &ForeignPeerState) {
        println!("State changed from {:?} to {:?}", old_state, new_state);
    }
}
