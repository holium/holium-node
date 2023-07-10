use std::fmt;

use async_trait::async_trait;

use crate::{action::Action, effect::Effect, OS};

use self::{booting::Booting, genesis::Genesis, idle::Idle, online::Online, shutdown::Shutdown};
pub mod booting;
pub mod genesis;
pub mod idle;
pub mod online;
pub mod shutdown;

pub type ActionResult = Result<Vec<Effect>, Box<dyn std::error::Error + Send>>;
pub type StateBox = Box<dyn State + Send>;

#[derive(Clone)]
pub enum KernelState {
    Genesis(Genesis),
    Booting(Booting),
    Online(Online),
    Idle(Idle),
    Shutdown(Shutdown),
}

#[async_trait(?Send)]
impl State for KernelState {
    async fn perform_action(&self, action: Action, os_state: &OS) -> ActionResult {
        match self {
            KernelState::Genesis(state) => state.perform_action(action, os_state).await,
            KernelState::Booting(state) => state.perform_action(action, os_state).await,
            KernelState::Online(state) => state.perform_action(action, os_state).await,
            KernelState::Idle(state) => state.perform_action(action, os_state).await,
            KernelState::Shutdown(state) => state.perform_action(action, os_state).await,
        }
    }

    fn describe(&self) -> String {
        match self {
            KernelState::Genesis(state) => state.describe(),
            KernelState::Booting(state) => state.describe(),
            KernelState::Online(state) => state.describe(),
            KernelState::Idle(state) => state.describe(),
            KernelState::Shutdown(state) => state.describe(),
        }
    }
}

impl CloneState for KernelState {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}

// Base trait for all states
#[async_trait(?Send)]
pub trait State: CloneState {
    async fn perform_action(&self, action: Action, os_state: &OS) -> ActionResult;
    fn describe(&self) -> String;
}

pub trait CloneState {
    fn clone_box(&self) -> StateBox;
}

impl Clone for StateBox {
    fn clone(&self) -> StateBox {
        self.clone_box()
    }
}

impl<T: 'static + State + fmt::Debug + Clone + Send> CloneState for T {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}

impl fmt::Debug for StateBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.describe().fmt(f)
    }
}
