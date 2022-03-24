pub mod conversions;
pub mod frozen;
pub mod index;
pub mod reference;

use serde::{Deserialize, Serialize};

pub use conversions::indices_to_refs;

pub const MAX_STATES: usize = 16;
pub const MAX_CHECKS_PER_STATE: usize = 3;
pub const MAX_COMMANDS_PER_STATE: usize = 3;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Seconds(pub ordered_float::NotNan<f32>);

impl Seconds {
    /// Creates a new Seconds wrapper from the given number of seconds
    ///
    /// # Panics
    ///
    /// If `seconds` is Nan
    pub fn new(seconds: f32) -> Self {
        Self(ordered_float::NotNan::new(seconds).unwrap())
    }
}

impl std::fmt::Display for Seconds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}s", self.0))
    }
}

/// Describes the check for a `native' condition, I.E, a condition that the state machine emulates.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub struct NativeFlagCondition(pub bool);

impl PartialEq<bool> for NativeFlagCondition {
    fn eq(&self, other: &bool) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub struct PyroContinuityCondition(pub bool);

impl PartialEq<bool> for PyroContinuityCondition {
    fn eq(&self, other: &bool) -> bool {
        self.0 == *other
    }
}

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

impl CheckData {
    pub fn kind(&self) -> CheckKind {
        match self {
            CheckData::Altitude(_) => CheckKind::Altitude,
            CheckData::ApogeeFlag(_) => CheckKind::Altitude,
            CheckData::Pyro1Continuity(_) => CheckKind::Pyro1Continuity,
            CheckData::Pyro2Continuity(_) => CheckKind::Pyro2Continuity,
            CheckData::Pyro3Continuity(_) => CheckKind::Pyro3Continuity,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CheckKind {
    Altitude,
    ApogeeFlag, //TODO: Maybe have a native flag variant with another enum for the kind of flag?
    Pyro1Continuity,
    Pyro2Continuity,
    Pyro3Continuity,
}

/// Represents the state that something's value can be, this can be the value a command will set
/// something to, or a value that a check will receive
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ObjectState {
    /// An On/Off True/False for a GPIO for example
    Bool(bool),
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

impl From<&reference::Command> for index::Command {
    fn from(c: &reference::Command) -> Self {
        Self {
            object: c.object,
            delay: c.delay,
        }
    }
}

/// An object that a command can act upon
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum CommandKind {
    Pyro1,
    Pyro2,
    Pyro3,
    Beacon,
    DataRate,
}

impl CommandKind {
    /// Adds a bool state to this `CommandKind`, assuming that is able to store a bool. This
    /// function will panic if self is `CommandKind::DataRate`, as the inner state data type for
    /// this is u16
    pub fn with_bool(self, val: bool) -> CommandObject {
        match self {
            CommandKind::Pyro1 => CommandObject::Pyro1(val),
            CommandKind::Pyro2 => CommandObject::Pyro2(val),
            CommandKind::Pyro3 => CommandObject::Pyro3(val),
            CommandKind::Beacon => CommandObject::Beacon(val),
            CommandKind::DataRate => panic!("cannot add bool to self when self is a DataRate"),
        }
    }

    pub fn with_u16(self, val: u16) -> CommandObject {
        let msg = match self {
            CommandKind::Pyro1 => "pyro1",
            CommandKind::Pyro2 => "pyro2",
            CommandKind::Pyro3 => "pyro3",
            CommandKind::Beacon => "beacon",
            CommandKind::DataRate => return CommandObject::DataRate(val),
        };
        panic!("cannot add u16 when self is an {msg}")
    }

    pub fn with_state(self, state: ObjectState) -> CommandObject {
        match state {
            ObjectState::Bool(val) => self.with_bool(val),
            ObjectState::Short(val) => self.with_u16(val),
            ObjectState::Float(_val) => todo!(),
        }
    }
}
