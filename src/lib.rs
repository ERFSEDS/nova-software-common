#![cfg_attr(not(feature = "std"), no_std)]

use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub struct ConfigFile {
    pub default_state: StateIndex,
    pub states: Vec<State, 16>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[repr(transparent)]
pub struct StateIndex(u8);

impl Into<usize> for StateIndex {
    fn into(self) -> usize {
        self.0 as usize
    }
}

/// A state that the rocket/flight computer can be in
///
/// This should be things like Armed, Stage1, Stage2, Safe, etc.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub name: String<16>,
    pub checks: Vec<Check, 3>,
    pub commands: Vec<Command, 1>,
    pub timeout: Option<Timeout>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timeout {
    pub time: f32,
    pub transition: StateIndex,
}

/// A check within a state that is run every time the state is run
#[derive(Debug, Serialize, Deserialize)]
pub struct Check {
    pub name: String<16>,
    pub check: CheckType,
    pub condition: CheckCondition,
    pub value: f32,
    pub on_satisfied: CheckSatisfied,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CheckType {
    Altitude,
    Pyro1Continuity,
    Pyro2Continuity,
    Pyro3Continuity,
}

/// Represents a type of state check
#[derive(Debug, Serialize, Deserialize)]
pub enum CheckCondition {
    FlagSet,
    FlagUnset,
    Equals { value: f32 },
    GreaterThan { value: f32 },
    LessThan { value: f32 },
    Between { upper_bound: f32, lower_bound: f32 },
}

/// A state transition due to a check being satisfied
/// This is how states transition from one to another.
///
/// The enum values are the indexes of states within the vector passed to StateMachine::from_vec()
#[derive(Debug, Serialize, Deserialize)]
pub enum CheckSatisfied {
    /// Represents a safe transition to another state
    Transition(StateIndex),
    /// Represents an abort to a safer state if an abort condition was met
    Abort(StateIndex),
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CommandObject {
    Pyro1,
    Pyro2,
    Pyro3,
    Beacon,
    DataRate,
}

/// A check within a state that is run every time the state is run
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    object: CommandObject,
    value: f32,
    delay: f32,
}

impl Command {
    pub fn get_pyro(&self) -> bool {
        self.value == 1.0
    }

    pub fn get_beacon(&self) -> bool {
        self.value == 1.0
    }
}
