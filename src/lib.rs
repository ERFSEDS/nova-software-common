#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod index;

pub use index::{Index, StateIndex, CheckIndex, CommandIndex};

use serde::{Deserialize, Serialize};
use heapless::Vec;
use core::sync::atomic::AtomicBool;

pub const MAX_STATES: usize = 16;
pub const MAX_CHECKS: usize = 64;
pub const MAX_COMMANDS: usize = 32;

pub const MAX_CHECKS_PER_STATE: usize = 4;
pub const MAX_COMMANDS_PER_STATE: usize = 4;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Seconds(f32);

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

/// Represents the state that something's value can be, this can be the value a command will set
/// something to, or a value that a check will receive
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ObjectState {
    /// An On/Off True/False for a GPIO for example
    Flag(bool),
    /// A floating-point value
    Float(f32),
    // TODO: We may want to rename/remove this, but this was for the DataRate
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    /// The object that this command will act upon
    pub object: CommandObject,

    /// The new state that the command's object should be in after it is executed
    pub state: ObjectState,

    /// How long after the state activates to execute this command
    pub delay: Seconds,

    #[cfg(feature = "executing")]
    /// Indicates weather or not this command has ben executed yet
    pub was_executed: AtomicBool,
}

impl Command {
    pub fn new(object: CommandObject, state: ObjectState, delay: Seconds) -> Self {
        Self {
            object,
            state,
            delay,
            #[cfg(feature = "executing")]
            was_executed: AtomicBool::new(false),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub default_state: StateIndex,
    pub states: Vec<State, MAX_STATES>,
    pub checks: Vec<Check, MAX_CHECKS>,
    pub commands: Vec<Command, MAX_COMMANDS>,
}

/// A state that the rocket/flight computer can be in
///
/// This should be things like Armed, Stage1, Stage2, Safe, etc.
///
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct State {
    //pub name: String<16>,
    pub checks: Vec<CheckIndex, MAX_CHECKS_PER_STATE>,
    pub commands: Vec<CommandIndex, MAX_COMMANDS_PER_STATE>,
    pub timeout: Option<Timeout>,
}

impl State {
    pub fn new(
        checks: Vec<CheckIndex, MAX_CHECKS_PER_STATE>,
        commands: Vec<CommandIndex, MAX_COMMANDS_PER_STATE>,
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

#[test]
fn test() {
    assert_eq!(core::mem::size_of::<ConfigFile>(), 0);
}
