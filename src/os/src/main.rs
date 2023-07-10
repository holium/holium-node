// use conduit::manager::ConduitManager;
use event_log::Event;
use global_state::GlobalState;

use action::{Action, BootArgs, PrebootArgs};
use effect::Effect;
use holon::OurPeer;
use modules::passport::{
    contracts::{azimuth::Azimuth, IdentitySystem},
    PassportModule,
};
use states::genesis::Genesis;

mod action;
mod effect;
mod event_log;
mod global_state;
pub mod holon;
pub mod modules;
pub mod states;

pub struct OSModules {
    pub passport: PassportModule,
    // pub conduit: ConduitManager,
}
pub struct TransitionResult {
    new_state: Box<dyn states::State>,
    effects: Vec<Effect>,
}
pub struct OS {
    modules: OSModules,
    node_state: Box<dyn states::State>,
    global_state: GlobalState,
    event_log: Vec<Event>,
}

impl OS {
    /// new: setup the preboot state of the OS
    pub fn new() -> OS {
        let initial_state = Genesis::new();
        OS {
            modules: OSModules {
                passport: PassportModule::new(),
                // conduit: ConduitManager::new(),
            },
            node_state: Box::new(initial_state),
            global_state: GlobalState::genesis(),
            event_log: vec![],
        }
    }

    /// genesis: initializes the OS
    /// Example
    /// ```
    /// let mut os = OS::new();
    /// os.genesis();
    /// ```
    ///
    pub async fn genesis(&mut self, hid: String, boot_key: String, bind_address: String) {
        let genesis_actions = vec![Action::Preboot(PrebootArgs {
            hid: hid,
            boot_key: boot_key,
            bind_address: bind_address,
            identity_system: IdentitySystem::Azimuth(Azimuth {
                contract: "0x223c067f8cf28ae173ee5cafea60ca44c335fecb".to_string(),
                network: "mainnet".to_string(),
            }),
        })];

        for action in genesis_actions {
            let transition = self.compute_transition(action).await;
            self.apply_transition(transition);
        }
    }

    /// boot: starts the main loop of the OS
    /// Example
    /// ```
    /// let mut os = OS::new();
    /// let port: Option<i16> = Some(9030);
    /// os.boot(port);
    /// ```
    ///     
    pub async fn boot(&mut self, port: Option<i16>) {
        let transition = self
            .compute_transition(Action::Boot(BootArgs { port }))
            .await;

        self.apply_transition(transition);

        // loop {
        //     let action = self.get_call_stack();
        //     let transition = self.compute_transition(action).await;
        //     self.apply_transition(transition);
        // }
    }

    // pub fn get_call_stack(&self) -> Vec<Action> {
    //     self.node_state.get_call_stack()
    // }

    pub fn push_event(&mut self, event: Event) {
        self.event_log.push(event);
    }

    // queue the action to be performed

    pub async fn compute_transition(&mut self, action: Action) -> TransitionResult {
        // Log the action
        self.push_event(Event::Action(action.clone()));

        let effects = self.node_state.perform_action(action, self).await.unwrap();

        // Extract new state from effects if it exists
        let new_state = effects.iter().find_map(|effect| match &effect {
            Effect::StateTransition(next_state) => Some((*next_state).clone_box()),
            _ => None,
        });

        match new_state {
            Some(state) => TransitionResult {
                new_state: state,
                effects,
            },
            None => TransitionResult {
                new_state: self.node_state.clone_box(), // no state transition
                effects,
            },
        }
    }

    pub fn apply_transition(&mut self, transition: TransitionResult) {
        println!(
            "transition: {} -> {}",
            self.node_state.describe(),
            transition.new_state.describe()
        );
        self.node_state = transition.new_state.clone_box();

        for effect in &transition.effects {
            // log the effect
            self.event_log.push(Event::Effect(effect.clone()));
            match effect {
                Effect::StateTransition(ref next_state) => {
                    self.event_log
                        .push(Event::StateTransition(format!("{:?}", next_state)));
                }
                Effect::GlobalStateChange(ref change) => {
                    self.global_state.apply_change(&change.clone());
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // TODO get CLI args
    let mut holon = OS::new();
    holon.genesis(
        "~lomder-librun".to_string(),
        "48244c1ea201f0c9c273b5171fd81e927ba365f6cb25d50a71c0049f2bb83dbb1d458968596166bafd3e732e95732903d7c4f10b61c3d3557ad5eb190f0bb212".to_string(),
        "/1/127.0.0.1:9030".to_string()
    )
    .await;

    let port: Option<i16> = Some(9030);
    holon.boot(port).await;

    let identity: OurPeer = holon.global_state.modules.passport.identity.unwrap();
    println!("{}", identity.get_addr());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_os() {
        let mut holon = OS::new();
        let mut holon2 = OS::new();
        holon.genesis(
            "~lomder-librun".to_string(),
            "48244c1ea201f0c9c273b5171fd81e927ba365f6cb25d50a71c0049f2bb83dbb1d458968596166bafd3e732e95732903d7c4f10b61c3d3557ad5eb190f0bb212".to_string(),
            "/1/127.0.0.1:9031".to_string()
        )
        .await;
        holon2.genesis(
            "~fasnut-famden".to_string(),
            "671216067cec38ddcf4f696663f4a504ac6bdf9899725547fac88560fe3ed3faed3e51a3f911cfd24a173a0ecf3998cf99be6368b34cbbd018a42092b607a8d2".to_string(),
            "/1/127.0.0.1:9032".to_string()
        )
        .await;

        let port: Option<i16> = Some(9031);
        let port2: Option<i16> = Some(9032);
        holon.boot(port).await;
        holon2.boot(port2).await;

        // let identity: OurPeer = holon.global_state.modules.passport.identity.unwrap();
        // let identity2: OurPeer = holon2.global_state.modules.passport.identity.unwrap();
    }
}
