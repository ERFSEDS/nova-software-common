use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub type ConfigFile = Vec<State, 16>;

/// A state that the rocket/flight computer can be in
///
/// This should be things like Armed, Stage1, Stage2, Safe, etc.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    name: String<32>,
    checks: Vec<Check, 4>,
}

impl State {
    pub fn new(name: String<32>, checks: Vec<Check, 4>) -> Self {
        Self { name, checks }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn checks(&self) -> &Vec<Check, 4> {
        &self.checks
    }
}

/// A check within a state that is run every time the state is run
#[derive(Debug, Serialize, Deserialize)]
pub struct Check {
    name: String<32>,
    value: String<32>,
    check_type: CheckType,
    satisfied: Option<CheckSatisfied>,
}

impl Check {
    pub fn new(
        name: String<32>,
        value: String<32>,
        check_type: CheckType,
        satisfied: Option<CheckSatisfied>,
    ) -> Self {
        Self {
            name,
            value,
            check_type,
            satisfied,
        }
    }
}

/// Represents a type of state check
#[derive(Debug, Serialize, Deserialize)]
pub enum CheckType {
    Flag,
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
    Transition(usize),
    /// Represents an abort to a safer state if an abort condition was met
    Abort(usize),
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
