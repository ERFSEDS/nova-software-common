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

impl From<StateIndex> for usize {
    fn from(index: StateIndex) -> Self {
        index.0 as usize
    }
}

/// A state that the rocket/flight computer can be in
///
/// This should be things like Armed, Stage1, Stage2, Safe, etc.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    //pub name: String<16>,
    pub checks: Vec<Check, 3>,
    pub commands: Vec<Command, 1>,
    pub timeout: Option<Timeout>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timeout {
    /// Time in seconds to wait before transitioning
    pub time: f32,
    /// The state to transition to
    pub transition: StateIndex,
}

/// A check within a state that is run every time the state is run
#[derive(Debug, Serialize, Deserialize)]
pub struct Check {
    //pub name: String<16>,
    pub check: CheckType,
    pub condition: CheckCondition,
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
pub enum CommandKind {
    Pyro1(bool),
    Pyro2(bool),
    Pyro3(bool),
    Beacon(bool),
    DataRate(u16),
}

/// A task that is run when the containing state is activated
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    /// The kind of command this is
    kind: CommandKind,

    /// How long after the state activates to execute this command
    delay: f32,
}

impl Command {
    pub fn get_pyro(&self) -> bool {
        match self.kind {
            CommandKind::Pyro1(val) => val,
            CommandKind::Pyro2(val) => val,
            CommandKind::Pyro3(val) => val,
            CommandKind::Beacon(val) => false,
            CommandKind::DataRate(_) => false,
        }
    }

    pub fn get_beacon(&self) -> bool {
        match self.kind {
            CommandKind::Pyro1(_) => false,
            CommandKind::Pyro2(_) => false,
            CommandKind::Pyro3(_) => false,
            CommandKind::Beacon(val) => val,
            CommandKind::DataRate(_) => false,
        }
    }
}
