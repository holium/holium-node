use std::collections::HashMap;

// Define the different states
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum State {
    Boot,
    Runtime,
    Error,
}

// Define the different events that can occur
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Event {
    BootFinished,
    RuntimeError,
    ErrorFixed,
}

struct StateMachine {
    current_state: State,
    transitions: HashMap<(State, Event), State>,
}

impl StateMachine {
    fn new() -> Self {
        let mut transitions = HashMap::new();

        transitions.insert((State::Boot, Event::BootFinished), State::Runtime);
        transitions.insert((State::Runtime, Event::RuntimeError), State::Error);
        transitions.insert((State::Error, Event::ErrorFixed), State::Runtime);

        StateMachine {
            current_state: State::Boot,
            transitions,
        }
    }

    fn process_event(&mut self, event: Event) {
        if let Some(&new_state) = self.transitions.get(&(self.current_state, event)) {
            self.current_state = new_state;
        } else {
            println!("Invalid event for current state: {:?}", event);
        }
    }

    fn print_state(&self) {
        println!("Current state: {:?}", self.current_state);
    }
}

fn main() {
    let mut state_machine = StateMachine::new();

    state_machine.print_state(); // Should print "Current state: Boot"

    state_machine.process_event(Event::BootFinished);
    state_machine.print_state(); // Should print "Current state: Runtime"

    state_machine.process_event(Event::RuntimeError);
    state_machine.print_state(); // Should print "Current state: Error"

    state_machine.process_event(Event::ErrorFixed);
    state_machine.print_state(); // Should print "Current state: Runtime"
}

// write a test for the state machine
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine() {
        let mut state_machine = StateMachine::new();

        assert_eq!(state_machine.current_state, State::Boot);

        state_machine.process_event(Event::BootFinished);
        assert_eq!(state_machine.current_state, State::Runtime);

        state_machine.process_event(Event::RuntimeError);
        assert_eq!(state_machine.current_state, State::Error);

        state_machine.process_event(Event::ErrorFixed);
        assert_eq!(state_machine.current_state, State::Runtime);
    }
}
