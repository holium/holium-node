use std::any::Any;
use std::fmt;

use crate::{action::Action, effect::Effect, OS};

pub trait State: CloneState {
    fn perform_action(&self, action: Action, os_state: &OS) -> Vec<Effect>;
}

pub trait CloneState {
    fn clone_box(&self) -> Box<dyn State>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn State> {
    fn clone(&self) -> Box<dyn State> {
        self.clone_box()
    }
}

impl<T: 'static + State + fmt::Debug + Clone> CloneState for T {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// impl dyn State {
//     fn as_any(&self) -> &dyn Any;
// }

impl fmt::Debug for Box<dyn State> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Attempt to downcast to the concrete state type
        if let Some(state) = self.as_any().downcast_ref::<Self>() {
            write!(f, "{:?}", state)
        } else {
            write!(f, "<unknown>")
        }
    }
}

// impl fmt::Debug for dyn State {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self.clone_box())
//     }
// }
