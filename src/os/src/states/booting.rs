use std::error::Error;
use std::fmt;

use crate::action::Action;
use crate::effect::Effect;
use crate::holon::OurPeer;
use crate::states::{CloneState, State};
use crate::OS;
use async_trait::async_trait;

use super::online::Online;
use super::{ActionResult, StateBox};

use anyhow::{bail, Result as GenericResult};

#[derive(Debug)]
enum BootingError {
    StartDatabase(String),
}

impl Error for BootingError {}

impl fmt::Display for BootingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BootingError::StartDatabase(message) => write!(f, "boot: error. {}", message),
        }
    }
}

// type BootingErrorBox = Box<dyn Error + Send>;

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

    pub fn start_conduit(&self, _identity: OurPeer) {
        println!("{} - starting conduit", self.describe());
        // spawn_conduit_listener(identity);
    }

    pub fn start_db(&self) -> GenericResult<()> {
        println!("{} - starting db", self.describe());

        if crate::modules::db::start().is_err() {
            bail!("{} - db::start failed", self.describe());
        }

        Ok(())
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

                // attempt to open a :memory: connection to SQLite, run a series of
                //  scripts found in the os/src/modules/db/sql folder, and import initial
                //  data for all registered agents
                if self.start_db().is_err() {
                    return ActionResult::Err(Box::new(BootingError::StartDatabase(
                        "start_db call failed".to_string(),
                    )));
                }

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
