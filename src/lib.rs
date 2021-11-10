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
    pub name: String<32>,
    pub checks: Vec<Check, 4>,
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
    pub name: String<20>,
    pub value: String<20>,
    pub condition: CheckCondition,
    pub satisfied: CheckSatisfied,
}

/// Represents a type of state check
#[derive(Debug, Serialize, Deserialize)]
pub enum CheckCondition {
    FlagSet,
    FlagUnSet,
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

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CommandObject {
    pyro1,
    pyro2,
    pyro3,
    beacon,
    data_rate,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CommandObjectValueType {
    Float32,
    Int32,
    Bool,
    GPIO,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CommandObjectValue {
    Float32(f32),
    Int32(i32),
    Bool(bool),
    GPIO(bool),
}

impl CommandObjectValue {
    /// Returns the type of the contained value
    pub fn classify(&self) -> CommandObjectValueType {
        match *self {
            CommandObjectValue::Float32(_) => CommandObjectValueType::Float32,
            CommandObjectValue::Int32(_) => CommandObjectValueType::Int32,
            CommandObjectValue::Bool(_) => CommandObjectValueType::Bool,
            CommandObjectValue::GPIO(_) => CommandObjectValueType::GPIO,
        }
    }
}

impl CommandObject {
    /// Returns the storage type of the given `CommandObject`
    pub fn get_type(&self) -> CommandObjectValueType {
        match *self {
            CommandObject::pyro1 => CommandObjectValueType::GPIO,
            CommandObject::pyro2 => CommandObjectValueType::GPIO,
            CommandObject::pyro3 => CommandObjectValueType::GPIO,
            CommandObject::beacon => CommandObjectValueType::Bool,
            CommandObject::data_rate => CommandObjectValueType::Int32,
        }
    }
}

/// A check within a state that is run every time the state is run
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    object: CommandObject,
    value: CommandObjectValue,
    delay: f32,
}

//Using fail instead of a bunch of calls to unreachable! or panic!
//leads to a much smaller code size
#[cold]
fn fail() -> ! {
    panic!("Tried to take object of wrong type!");
}

impl Command {
    pub fn get_pyro(&self) -> bool { 
        match self.object {
            CommandObject::pyro1 | CommandObject::pyro2 | CommandObject::pyro3 => {
                match self.value {
                    CommandObjectValue::GPIO(v) => v,
                    _ => fail(),
                }
            }
            _ => fail(),
        }
    }

    pub fn get_beacon(&self) -> bool {
        match self.object {
            CommandObject::beacon => match self.value {
                CommandObjectValue::Bool(v) => v,
                _ => fail(),
            },
            _ => fail(),
        }
    }

    pub fn get_data_rate(&self) -> i32 {
        match self.object {
            CommandObject::beacon => match self.value {
                CommandObjectValue::Int32(v) => v,
                _ => fail(),
            },
            _ => fail(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //Checks for size based breaking changes
    #[test]
    fn test_config_size() {
        let size = std::mem::size_of::<ConfigFile>();
        //TODO: This is massive.
        //We need to try to optimize this
        assert_eq!(size, 7944);
    }
}
