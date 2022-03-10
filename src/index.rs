//! State machine data structures that use indices to reference state transitions.
//! This is needed when the config file is serialized between the verifier and the flight computer.

use crate::{MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES};

use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConfigFile {
    pub default_state: StateIndex,
    pub states: Vec<State, MAX_STATES>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
/// The which references a particural state
pub struct StateIndex(u8);

impl StateIndex {
    /// Creates a new `StateIndex` without checking that `index` is valid. The validity of `index`
    /// is based on the context in which it is used, therefore we provide no safe new function. The
    /// caller must take the responsibility that this new `StateIndex` makes senes for its own use case
    ///
    /// # Safety
    /// The caller must guarntee that index is valid within the context of the value it is
    /// referencing
    ///
    /// Note: Passing in an invalid index here will never lead to memory unsafety.
    /// This wrapper simply allows us to feel better about unwrapping `get()`s that use index at
    /// other places in the codebase because we assume constructing an invalid `StateIndex` is
    /// impossible
    pub unsafe fn new_unchecked(index: u8) -> Self {
        StateIndex(index)
    }
}

impl From<StateIndex> for usize {
    fn from(index: StateIndex) -> Self {
        index.0 as usize
    }
}

/// A state that the rocket/flight computer can be in
///
/// This should be things like Armed, Stage1, Stage2, Safe, etc.
///
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct State {
    //pub name: String<16>,
    pub checks: Vec<Check, MAX_CHECKS_PER_STATE>,
    pub commands: Vec<Command, MAX_COMMANDS_PER_STATE>,
    pub timeout: Option<Timeout>,
}

impl State {
    pub fn new(
        checks: Vec<Check, MAX_CHECKS_PER_STATE>,
        commands: Vec<Command, MAX_COMMANDS_PER_STATE>,
        timeout: Option<Timeout>,
    ) -> Self {
        Self {
            checks,
            commands,
            timeout,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Timeout {
    /// Time in seconds to wait before transitioning
    pub time: f32,
    /// The transition that is made when the state times out
    pub transition: StateTransition,
}

impl Timeout {
    pub fn new(time: f32, transition: StateTransition) -> Self {
        Self { time, transition }
    }
}

/// A check within a state that is run every time the state is run
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Check {
    //pub name: String<16>,
    pub data: crate::CheckData,
    pub transition: Option<StateTransition>,
}

impl Check {
    pub fn new(data: crate::CheckData, transition: Option<StateTransition>) -> Self {
        Self { data, transition }
    }
}

/// An action that takes place at a specific time after the state containing this is entered
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Command {
    /// The object that this command will act upon
    pub object: crate::CommandObject,

    /// How long after the state activates to execute this command
    pub delay: crate::Seconds,
}

impl Command {
    pub fn new(object: crate::CommandObject, delay: crate::Seconds) -> Self {
        Self { object, delay }
    }
}

/// A state transition due to a check being satisfied
/// This is how states transition from one to another.
///
/// The enum values are the indexes of a state
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum StateTransition {
    /// Represents a safe transition to another state
    Transition(StateIndex),
    /// Represents an abort to a safer state if an abort condition was met
    Abort(StateIndex),
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_pointer_width = "32")]
    fn test() {
        assert_eq!(core::mem::size_of::<crate::index::ConfigFile>(), 1608);
    }
}
