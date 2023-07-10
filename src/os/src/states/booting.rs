use crate::action::Action;
use crate::effect::Effect;
use crate::holon::OurPeer;
use crate::states::{CloneState, State};
use crate::OS;
use async_trait::async_trait;

use super::online::Online;
use super::{ActionResult, StateBox};

#[derive(Clone)]
pub struct Booting;

/// # Booting
/// Booting is the first state of the system after genesis.
/// It is responsible for starting the networking stack and booting the system services.
///
impl Booting {
    pub fn new() -> Booting {
        Booting
    }

    pub fn start_conduit(&self, identity: OurPeer) {
        println!("{} - starting conduit", self.describe());
        // spawn_conduit_listener(identity);
    }

    pub fn start_db(&self) {
        println!("{} - starting db", self.describe());
    }

    pub fn start_client_server(&self) {
        println!("{} - starting client server", self.describe());
    }
}

#[async_trait(?Send)]
impl State for Booting {
    fn describe(&self) -> String {
        String::from("booting")
    }

    async fn perform_action(&self, action: Action, os_state: &OS) -> ActionResult {
        match action {
            Action::Boot(_) => {
                let identity = os_state
                    .global_state
                    .modules
                    .passport
                    .identity
                    .clone()
                    .unwrap();

                self.start_conduit(identity);

                ActionResult::Ok(vec![Effect::StateTransition(Box::new(Online::new()))])
            }
            _ => ActionResult::Ok(vec![]),
        }
    }
}

impl CloneState for Booting {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}
