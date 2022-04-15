use super::traits::{GenericTimestamp, Timestamp};

use crate::control::Controls;
use crate::data_acquisition::DataWorkspace;
use novafc_config_format::{
    reference::{Check, Command, State, StateTransition},
    CheckData, Value,
};
use novafc_config_format::{FloatCondition, Seconds};

pub struct StateMachine<'state> {
    current_state: &'state State<'state>,
    start_time: Timestamp,

    /// The instant the last state was activated
    last_transition_time: Timestamp,
}

impl<'state> StateMachine<'state> {
    pub fn new(begin: &'state State<'state>) -> Self {
        let time = Timestamp::now();

        #[cfg(feature = "std")]
        println!("State machine starting in state: {}", begin.id);

        Self {
            current_state: begin,
            start_time: time,
            last_transition_time: time,
        }
    }

    /// Executes one step of the state machine
    /// Commands of the current state are executed, checks are checked and if the conditions are
    /// met, transitions to a new state
    ///
    /// This function should be called repeatly at a rapid rate
    pub fn execute(&mut self) {
        if let Some(transition) = self.execute_state() {
            self.transition(transition);
        }
    }

    /// Executes the current state, returning the next state to transition to, if any
    fn execute_state(&mut self) -> Option<StateTransition<'state>> {
        // Execute commands
        for command in self.current_state.commands.iter() {
            self.execute_command(command);
        }

        // Execute checks
        for check in self.current_state.checks.iter() {
            if let Some(transition) = self.execute_check(check) {
                return Some(transition);
            }
        }

        // Check for timeout
        if let Some(timeout) = &self.current_state.timeout.get() {
            // Checks if the state has timed out
            if self.start_time.elapsed() >= timeout.time {
                Some(timeout.transition)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn execute_command(&mut self, command: &Command) {
        if !command.was_executed.get() {
            if self.last_transition_time.elapsed() >= command.delay {
                // FIXME
                //self.controls.set(command.object);
                command.was_executed.set(true);
            }
        }
    }

    fn execute_check(&self, check: &Check<'state>) -> Option<StateTransition<'state>> {
        // FIXME
        /*
        let value = self.data_workspace.get_object(check.data.kind());

        let satisfied = match (check.data, value) {
            (CheckData::ApogeeFlag(expected), Value::Bool(actual)) => expected == actual,
            (CheckData::Altitude(condition), Value::F32(actual)) => match condition {
                FloatCondition::LessThan(expected) => actual < expected,
                FloatCondition::GreaterThan(expected) => actual > expected,
                FloatCondition::Between {
                    upper_bound,
                    lower_bound,
                } => (actual >= upper_bound && actual <= lower_bound),
            },
            (CheckData::Pyro1Continuity(expected), Value::Bool(actual))
            | (CheckData::Pyro2Continuity(expected), Value::Bool(actual))
            | (CheckData::Pyro3Continuity(expected), Value::Bool(actual)) => expected == actual,
            // Unreachable here since there would have to be a bug inside data workspace which
            // always returns the same type for a given CheckKind enum, so this would be found
            // deterministically in testing
            _ => unreachable!(
                "mismatched types while executing check with {:?} vs {:?}",
                check.data, value
            ),
        };

        satisfied.then(|| check.transition).flatten()
        */
        None
    }

    fn transition(&mut self, transition: StateTransition<'state>) {
        let new_state = match transition {
            StateTransition::Abort(state) => {
                #[cfg(feature = "std")]
                println!(
                    "[{}s] Aborted to state: {}",
                    self.start_time.elapsed(),
                    state.id
                );
                // Here we would have abort reporting of some kind like some "callback" to the data
                // acquisition module
                state
            }
            StateTransition::Transition(state) => {
                #[cfg(feature = "std")]
                println!(
                    "[{}] Transitioned to state: {}",
                    self.start_time.elapsed(),
                    state.id
                );
                // We may also put some kind of transition reporting here or just use state ID's
                state
            }
        };

        // Set the new state and reset the state time
        self.current_state = new_state;
        self.last_transition_time = Timestamp::now();
    }
}

pub struct Timeout<'a> {
    pub time: Seconds,
    pub transition: StateTransition<'a>,
}

impl<'a> Timeout<'a> {
    pub fn new(time: Seconds, transition: StateTransition<'a>) -> Self {
        Self { time, transition }
    }
}
