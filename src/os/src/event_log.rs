use crate::{action::Action, effect::Effect};

#[derive(Clone, Debug)]
pub enum Event {
    Action(Action),
    Effect(Effect),
    StateTransition(String),
}
