//! State machine data structures that use indices to reference state transitions. 
//! This is needed when the config file is serialized between the verifier and the flight computer.

use crate::{MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES};
use crate::reference;

use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConfigFile {
    pub default_state: StateIndex,
    pub states: Vec<State, MAX_STATES>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct StateIndex(u8);

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
    pub commands: Vec<crate::Command, MAX_COMMANDS_PER_STATE>,
    pub timeout: Option<Timeout>,
}

impl State {
    pub fn new(
        checks: Vec<Check, MAX_CHECKS_PER_STATE>,
        commands: Vec<crate::Command, MAX_COMMANDS_PER_STATE>,
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
    pub object: crate::CheckObject,
    pub condition: crate::CheckCondition,
    pub transition: StateTransition,
}

impl Check {
    pub fn new(
        object: crate::CheckObject,
        condition: crate::CheckCondition,
        transition: StateTransition,
    ) -> Self {
        Self {
            object,
            condition,
            transition,
        }
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
