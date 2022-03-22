#![cfg_attr(not(feature = "std"), no_std)]

use core::sync::atomic::AtomicBool;
use std::time::SystemTime;

use control::Controls;
use data_acquisition::DataWorkspace;
use data_format::{
    CheckData, CommandObject, ObjectState, MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE,
};
use heapless::Vec;

pub struct StateMachine<'a, 'b, 'c> {
    current_state: &'a State<'a>,
    start_time: SystemTime,
    state_time: SystemTime,
    data_workspace: &'b DataWorkspace,
    controls: &'c mut Controls,
}

impl<'a, 'b, 'c> StateMachine<'a, 'b, 'c> {
    pub fn new(
        begin: &'a State<'a>,
        data_workspace: &'b DataWorkspace,
        controls: &'c mut Controls,
    ) -> Self {
        let time = SystemTime::now();

        #[cfg(feature = "std")]
        println!("State machine starting in state: {}", begin.id);

        Self {
            current_state: begin,
            start_time: time,
            state_time: time,
            data_workspace,
            controls,
        }
    }

    pub fn execute(&mut self) {
        if let Some(transition) = self.execute_state() {
            self.transition(transition);
        }
    }

    fn execute_state(&mut self) -> Option<StateTransition<'a>> {
        // Execute commands
        for command in self.current_state.commands.iter() {
            self.execute_command(command);
        }

        // Execute checks
        for check in self.current_state.checks.iter() {
            if let Some(transition) = self.execute_check(check) {
                return Some(transition);
            }
        }

        // Check for timeout
        if let Some(timeout) = &self.current_state.timeout {
            // Checks if the state has timed out
            if self.state_time.elapsed().unwrap().as_secs_f32() >= timeout.time {
                Some(timeout.transition)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn execute_command(&mut self, command: &Command) {
        if !command
            .was_executed
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            if self.state_time.elapsed().unwrap().as_secs_f32() >= command.delay {
                self.controls.set(command.object, command.setting);
                command
                    .was_executed
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    fn execute_check(&self, check: &Check<'a>) -> Option<StateTransition<'a>> {
        let value = self.data_workspace.get_object(check.data.);

        let satisfied = match check.data {
            CheckData::ApogeeFlag(flag) => {
                todo!();
            }
            CheckData::Altitude(altitude) => {
                todo!();
            }
            CheckData::Pyro1Continuity(cont)
            | CheckData::Pyro2Continuity(cont)
            | CheckData::Pyro3Continuity(cont) => {
                todo!();
            } /*CheckCondition::FlagSet | CheckCondition::FlagUnset => match value {
                  ObjectState::Flag(b) => b == matches!(check.condition, CheckCondition::FlagSet),
                  _ => panic!(
                      "{}",
                      if cfg!(feature = "std") {
                          "Non-flag value provided to a check that requires a FlagSet/Unset"
                      } else {
                          ""
                      }
                  ),
              },
              CheckCondition::LessThan { value: other } => match value {
                  ObjectState::Float(f) => f < other,
                  _ => panic!(
                      "{}",
                      if cfg!(feature = "std") {
                          "Non-float value provided to a check that requires a float value (LessThan)"
                      } else {
                          ""
                      }
                  ),
              },
              CheckCondition::GreaterThan { value: other } => match value {
                  ObjectState::Float(f) => f > other,
                  _ => panic!(
                      "{}",
                      if cfg!(feature = "std") {
                          "Non-float value provided to a check that requires a float value (GreaterThan)"
                      } else {
                          ""
                      }
                  ),
              },
              CheckCondition::Between {
                  upper_bound,
                  lower_bound,
              } => match value {
                  ObjectState::Float(f) => f < upper_bound && f > lower_bound,
                  _ => panic!(
                      "{}",
                      if cfg!(feature = "std") {
                          "Non-float value provided to a check that requires a float value (Between)"
                      } else {
                          ""
                      }
                  ),
              },
              */
        };

        satisfied.then(|| check.transition)
    }

    fn transition(&mut self, transition: StateTransition<'a>) {
        let new_state = match transition {
            StateTransition::Abort(state) => {
                #[cfg(feature = "std")]
                println!(
                    "[{}s] Aborted to state: {}",
                    self.start_time.elapsed().unwrap().as_secs_f32(),
                    state.id
                );
                // Here we would have abort reporting of some kind like some "callback" to the data
                // acquisition module
                state
            }
            StateTransition::Transition(state) => {
                #[cfg(feature = "std")]
                println!(
                    "[{}s] Transitioned to state: {}",
                    self.start_time.elapsed().unwrap().as_secs_f32(),
                    state.id
                );
                // We may also put some kind of transition reporting here or just use state ID's
                state
            }
        };

        // Set the new state and reset the state time
        self.current_state = new_state;
        self.state_time = SystemTime::now();
    }
}

pub struct Timeout<'a> {
    pub time: f32,
    pub transition: StateTransition<'a>,
}

impl<'a> Timeout<'a> {
    pub fn new(time: f32, transition: StateTransition<'a>) -> Self {
        Self { time, transition }
    }
}

pub struct State<'a> {
    pub id: u8,
    pub checks: Vec<&'a Check<'a>, MAX_CHECKS_PER_STATE>,
    pub commands: Vec<&'a Command, MAX_COMMANDS_PER_STATE>,
    pub timeout: Option<Timeout<'a>>,
}

impl<'a> State<'a> {
    pub fn new(
        id: u8,
        checks: Vec<&'a Check<'a>, MAX_CHECKS_PER_STATE>,
        commands: Vec<&'a Command, MAX_COMMANDS_PER_STATE>,
        timeout: Option<Timeout<'a>>,
    ) -> Self {
        Self {
            id,
            checks,
            commands,
            timeout,
        }
    }
}

pub struct Check<'a> {
    pub data: CheckData,
    pub transition: StateTransition<'a>,
}

impl<'a> Check<'a> {
    pub fn new(data: CheckData, transition: StateTransition<'a>) -> Self {
        Self { data, transition }
    }
}

#[derive(Copy, Clone)]
pub enum StateTransition<'a> {
    Transition(&'a State<'a>),
    Abort(&'a State<'a>),
}

pub struct Command {
    pub object: CommandObject,
    pub setting: ObjectState,
    pub delay: f32,
    pub was_executed: AtomicBool,
}

impl Command {
    pub fn new(object: CommandObject, setting: ObjectState, delay: f32) -> Self {
        Self {
            object,
            setting,
            delay,
            was_executed: AtomicBool::new(false),
        }
    }
}
