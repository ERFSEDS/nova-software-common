#![cfg_attr(not(feature = "std"), no_std)]

pub const MAX_STATES: usize = 16;
pub const MAX_CHECKS_PER_STATE: usize = 3;
pub const MAX_COMMANDS_PER_STATE: usize = 3;

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
    pub object: CheckObject,
    pub condition: CheckCondition,
    pub transition: StateTransition,
}

impl Check {
    pub fn new(
        object: CheckObject,
        condition: CheckCondition,
        transition: StateTransition,
    ) -> Self {
        Self {
            object,
            condition,
            transition,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum CheckObject {
    Altitude,
    Pyro1Continuity,
    Pyro2Continuity,
    Pyro3Continuity,
}

/// Represents a type of state check
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum CheckCondition {
    FlagSet,
    FlagUnset,
    // Equals { value: f32 },
    GreaterThan { value: f32 },
    LessThan { value: f32 },
    Between { upper_bound: f32, lower_bound: f32 },
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

/// Represents the state that something's value can be, this can be the value a command will set
/// something to, or a value that a check will receive
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ObjectState {
    /// An On/Off True/False for a GPIO for example
    Flag(bool),
    /// A floating-point value
    Float(f32),
    // We may want to rename/remove this, but this was for the DataRate
    Short(u16),
}

/// An object that a command can act upon
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum CommandObject {
    Pyro1,
    Pyro2,
    Pyro3,
    Beacon,
    DataRate,
}

/// An action that takes place at a specific time after the state containing this is entered
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Command {
    /// The object that this command will act upon
    pub object: CommandObject,

    /// The new state that the command's object should be in after it is executed
    pub state: ObjectState,

    /// How long after the state activates to execute this command (in seconds)
    pub delay: f32,
}

impl Command {
    pub fn new(object: CommandObject, state: ObjectState, delay: f32) -> Self {
        Self {
            object,
            state,
            delay,
        }
    }
}
