#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod index;

pub use index::{CheckIndex, CommandIndex, Index, StateIndex};

#[cfg(feature = "executing")]
use core::sync::atomic::{AtomicBool, Ordering};

use heapless::Vec;
use serde::{Deserialize, Serialize};

pub const MAX_STATES: usize = 16;
pub const MAX_CHECKS: usize = 64;
pub const MAX_COMMANDS: usize = 32;

pub const MAX_CHECKS_PER_STATE: usize = 4;
pub const MAX_COMMANDS_PER_STATE: usize = 4;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Seconds(pub f32);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConfigFile {
    pub default_state: StateIndex,
    pub states: Vec<State, MAX_STATES>,
    pub checks: Vec<Check, MAX_CHECKS>,
    pub commands: Vec<Command, MAX_COMMANDS>,
}

impl ConfigFile {
    pub fn get_state(&self, index: StateIndex) -> &State {
        let index = index.0.0 as usize;
        &self.states[index]
    }

    pub fn get_check(&self, index: CheckIndex) -> &Check {
        let index = index.0.0 as usize;
        &self.checks[index] 
    }

    pub fn get_command(&self, index: CheckIndex) -> &Command {
        let index = index.0.0 as usize;
        &self.commands[index]
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
    FlagEq(bool),
    // Equals { value: f32 },
    GreaterThan(f32),
    LessThan(f32),
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
    Pyro1(bool),
    Pyro2(bool),
    Pyro3(bool),
    Beacon(bool),
    DataRate(u16),
}

/// An action that takes place at a specific time after the state containing this is entered
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    /// The object that this command will act upon
    pub object: CommandObject,

    /// How long after the state activates to execute this command
    pub delay: Seconds,

    #[cfg(feature = "executing")]
    #[serde(skip)]
    /// Indicates weather or not this command has ben executed yet
    pub was_executed: AtomicBool,
}

impl Command {
    pub fn new(object: CommandObject, delay: Seconds) -> Self {
        Self {
            object,
            delay,
            #[cfg(feature = "executing")]
            was_executed: AtomicBool::new(false),
        }
    }
}

impl Clone for Command {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            delay: self.delay.clone(),
            #[cfg(feature = "executing")]
            was_executed: AtomicBool::new(self.was_executed.load(Ordering::SeqCst)),
        }
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        #[allow(unused_mut)] // Triggered when the #[cfg(...)] below is disabled
        let mut eq = self.object.eq(&other.object) && self.delay.eq(&other.delay);

        #[cfg(feature = "executing")]
        {
            eq &= self
                .was_executed
                .load(Ordering::SeqCst)
                .eq(&other.was_executed.load(Ordering::SeqCst));
        }
        eq
    }
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
