//! State machine data structures that use references to reference state transitions.
//! This module's types are uses as opposed to [`index`] during runtime, when being able to easily
//! reference a different state is important

#[cfg(feature = "executing")]
use core::sync::atomic::AtomicBool;

use crate::{MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES, Seconds};

use heapless::Vec;

pub struct ConfigFile<'s> {
    pub default_state: &'s State<'s>,
    pub states: Vec<&'s State<'s>, MAX_STATES>,
}

pub struct Timeout<'s> {
    pub time: f32,
    pub transition: StateTransition<'s>,
}

impl<'s> Timeout<'s> {
    pub fn new(time: f32, transition: StateTransition<'s>) -> Self {
        Self { time, transition }
    }
}

pub struct State<'s> {
    pub id: u8,
    pub checks: Vec<&'s Check<'s>, MAX_CHECKS_PER_STATE>,
    pub commands: Vec<&'s Command, MAX_COMMANDS_PER_STATE>,
    pub timeout: Option<Timeout<'s>>,
}

impl<'s> State<'s> {
    pub fn new(
        id: u8,
        checks: Vec<&'s Check<'s>, MAX_CHECKS_PER_STATE>,
        commands: Vec<&'s Command, MAX_COMMANDS_PER_STATE>,
        timeout: Option<Timeout<'s>>,
    ) -> Self {
        Self {
            id,
            checks,
            commands,
            timeout,
        }
    }
}

pub struct Check<'s> {
    pub object: crate::CheckObject,
    pub condition: crate::CheckCondition,
    pub transition: StateTransition<'s>,
}

impl<'s> Check<'s> {
    pub fn new(
        object: crate::CheckObject,
        condition: crate::CheckCondition,
        transition: StateTransition<'s>,
    ) -> Self {
        Self {
            object,
            condition,
            transition,
        }
    }
}

#[derive(Copy, Clone)]
pub enum StateTransition<'s> {
    Transition(&'s State<'s>),
    Abort(&'s State<'s>),
}

pub struct Command {
    pub object: crate::CommandObject,
    pub state: crate::ObjectState,
    pub delay: Seconds,
    #[cfg(feature = "executing")]
    pub was_executed: AtomicBool,
}

impl From<&crate::Command> for Command {
    fn from(c: &crate::Command) -> Self {
        Self {
            object: c.object,
            state: c.state,
            delay: c.delay,
            #[cfg(feature = "executing")]
            was_executed: AtomicBool::new(false),
        }
    }
}
