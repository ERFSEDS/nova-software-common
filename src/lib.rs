#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod conversions;
pub mod frozen;
pub mod index;
pub mod reference;

pub use conversions::indices_to_refs;

pub const MAX_STATES: usize = 16;
pub const MAX_CHECKS_PER_STATE: usize = 3;
pub const MAX_COMMANDS_PER_STATE: usize = 3;

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Command {
    /// The object that this command will act upon
    pub object: CommandObject,

    /// The new state that the command's object should be in after it is executed
    pub state: ObjectState,

    /// How long after the state activates to execute this command
    pub delay: Seconds,
}

impl Command {
    pub fn new(object: CommandObject, state: ObjectState, delay: Seconds) -> Self {
        Self {
            object,
            state,
            delay,
        }
    }
}

impl From<&crate::reference::Command> for Command {
    fn from(c: &crate::reference::Command) -> Self {
        Self {
            object: c.object,
            state: c.state,
            delay: c.delay,
        }
    }
}
