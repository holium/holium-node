// use conduit::manager::ConduitManager;
use event_log::Event;
use global_state::GlobalState;

use action::{Action, PrebootArgs};
use effect::{Effect, Effects};
use modules::passport::PassportModule;
use states::init::Init;

mod action;
mod effect;
mod event_log;
mod global_state;
mod modules;
pub mod state;
pub mod states;

pub struct OSModules {
    pub passport: PassportModule,
    // pub conduit: ConduitManager,
}

// type Reaction = Box<dyn Fn(&Effect) -> Option<Action>>;
pub struct TransitionResult {
    new_state: Box<dyn state::State>,
    effects: Vec<Effect>,
}
pub struct OS {
    current_state: Box<dyn state::State>,
    global_state: GlobalState,
    // reactions: Vec<Reaction>,
    event_log: Vec<Event>,
    modules: OSModules,
}

impl OS {
    pub fn new() -> OS {
        let initial_state = Init::new();
        OS {
            current_state: Box::new(initial_state),
            global_state: GlobalState::new(),
            // reactions: vec![],
            event_log: vec![],
            modules: OSModules {
                passport: PassportModule::new(),
                // conduit: ConduitManager::new(),
            },
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.event_log.push(event);
    }

    // queue the action to be performed
    // pub fn

    pub fn compute_transition(&mut self, action: Action) -> TransitionResult {
        // Log the action
        self.push_event(Event::Action(action.clone()));

        let effects = self.current_state.perform_action(action, self);

        // Extract new state from effects if it exists
        let new_state = effects.iter().find_map(|effect| match &effect.effect {
            Effects::StateTransition(next_state) => Some((*next_state).clone_box()),
            _ => None,
        });

        match new_state {
            Some(state) => TransitionResult {
                new_state: state,
                effects,
            },
            None => TransitionResult {
                new_state: self.current_state.clone_box(), // no state transition
                effects,
            },
        }
    }

    pub fn apply_transition(&mut self, transition: TransitionResult) {
        self.current_state = transition.new_state;

        for effect in &transition.effects {
            // log the effect
            self.event_log.push(Event::Effect(effect.clone()));

            match effect.effect {
                Effects::StateTransition(ref next_state) => {
                    // Log the state transition
                    self.event_log
                        .push(Event::StateTransition(format!("{:?}", next_state)));
                }
                Effects::GlobalStateChange(ref change) => {
                    self.global_state.apply_change(&change.clone())
                }
                _ => {}
            }
        }
    }

    // pub fn transition(&mut self, action: Action) {
    //     // Log the action
    //     self.event_log.push(Event::Action(action.clone()));

    //     let effects = self.current_state.perform_action(action, self);

    //     for effect in &effects {
    //         self.event_log.push(Event::Effect(effect.clone()));
    //         match effect.effect {
    //             Effects::StateTransition(ref next_state) => {
    //                 // Log the state transition
    //                 self.event_log
    //                     .push(Event::StateTransition(format!("{:?}", next_state)));

    //                 self.current_state = next_state.clone();
    //             }
    //             Effects::GlobalStateChange(ref change) => {
    //                 self.global_state.apply_change(&change.clone())
    //             }
    //             _ => {}
    //         }

    //         // for reaction in &self.reactions {
    //         //     if let Some(action) = reaction(&effect) {
    //         //         self.transition(action);
    //         //     }
    //         // }
    //     }
    // }
}

fn main() {
    // TODO get CLI args
    let mut os = OS::new();

    // os.transition(Action::Preboot {
    //     path: String::from("os/start_boot"),
    //     data: PrebootArgs {
    //         hid: String::from("~lomder-librun"),
    //         boot_key: String::from("boot_key"),
    //     },
    // });
}
