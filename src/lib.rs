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
pub struct Seconds(pub f32);

/// Describes the check for a `native' condition, I.E, a condition that the state machine emulates.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct NativeFlagCondition(pub bool);

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct PyroContinuityCondition(pub bool);

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum FloatCondition {
    GreaterThan(f32),
    LessThan(f32),
    Between { upper_bound: f32, lower_bound: f32 },
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum CheckData {
    Altitude(FloatCondition),
    ApogeeFlag(NativeFlagCondition),
    Pyro1Continuity(PyroContinuityCondition),
    Pyro2Continuity(PyroContinuityCondition),
    Pyro3Continuity(PyroContinuityCondition),
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
    Pyro1(bool),
    Pyro2(bool),
    Pyro3(bool),
    Beacon(bool),
    DataRate(u16),
}

impl From<&crate::reference::Command> for index::Command {
    fn from(c: &crate::reference::Command) -> Self {
        Self {
            object: c.object,
            delay: c.delay,
        }
    }
}
