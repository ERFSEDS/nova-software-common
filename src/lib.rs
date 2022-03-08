#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod conversions;
mod index;
mod reference;

pub use conversions::indices_to_refs;

use heapless::Vec;
use serde::{Deserialize, Serialize};

#[cfg(feature = "executing")]
use core::sync::atomic::{AtomicBool, Ordering};

pub const MAX_STATES: usize = 16;
pub const MAX_CHECKS_PER_STATE: usize = 3;
pub const MAX_COMMANDS_PER_STATE: usize = 3;

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
    pub was_executed: AtomicBool,
}

impl Clone for Command {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            state: self.state.clone(),
            delay: self.delay.clone(),
            #[cfg(feature = "executing")]
            was_executed: AtomicBool::new(self.was_executed.load(Ordering::SeqCst)),
        }
    }
}

impl Command {
    #[cfg(feature = "executing")]
    pub fn new(
        object: CommandObject,
        state: ObjectState,
        delay: Seconds,
        was_executed: AtomicBool,
    ) -> Self {
        Self {
            object,
            state,
            delay,
            was_executed,
        }
    }

    #[cfg(not(feature = "executing"))]
    pub fn new(object: CommandObject, state: ObjectState, delay: Seconds) -> Self {
        Self {
            object,
            state,
            delay,
        }
    }
}

pub struct ConfigFile<State, RefType> {
    /// The default state of this config file. Always a member of `states`
    pub default_state: RefType,
    pub states: Vec<State, MAX_STATES>,
}

pub struct State<Check, Timeout> {
    pub id: u8,
    pub checks: Vec<Check, MAX_CHECKS_PER_STATE>,
    pub commands: Vec<Command, MAX_COMMANDS_PER_STATE>,
    pub timeout: Option<Timeout>,
}

impl<Check, Timeout> State<Check, Timeout> {
    pub fn new(
        id: u8,
        checks: Vec<Check, MAX_CHECKS_PER_STATE>,
        commands: Vec<Command, MAX_COMMANDS_PER_STATE>,
        timeout: Option<Timeout>,
    ) -> Self {
        Self {
            id,
            checks,
            commands,
            timeout,
        }
    }
}

pub struct Timeout<StateTy> {
    pub time: f32,
    pub transition: StateTransition<StateTy>,
}

impl<StateTy> Timeout<StateTy> {
    pub fn new(time: f32, transition: StateTransition<StateTy>) -> Self {
        Self { time, transition }
    }
}

pub struct Check<StateTy> {
    pub object: CheckObject,
    pub condition: CheckCondition,
    pub transition: StateTransition<StateTy>,
}

impl<StateTy> Check<StateTy> {
    pub fn new(
        object: CheckObject,
        condition: CheckCondition,
        transition: StateTransition<StateTy>,
    ) -> Self {
        Self {
            object,
            condition,
            transition,
        }
    }
}

#[derive(Copy, Clone)]
pub enum StateTransition<StateTy> {
    Transition(StateTy),
    Abort(StateTy),
}
