//! State machine data structures that use references to reference state transitions.
//! This module's types are uses as opposed to [`index`] during runtime, when being able to easily
//! reference a different state is important

#[cfg(feature = "executing")]
use core::sync::atomic::AtomicBool;

use core::cell::Cell;

use crate::{frozen::FrozenVec, Seconds, MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES};

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
    pub checks: FrozenVec<&'s Check<'s>, MAX_CHECKS_PER_STATE>,
    pub commands: FrozenVec<&'s Command, MAX_COMMANDS_PER_STATE>,
    pub timeout: Cell<Option<Timeout<'s>>>,
}

impl<'s> State<'s> {
    pub(crate) fn new(id: u8) -> Self {
        Self {
            id,
            checks: FrozenVec::new(),
            commands: FrozenVec::new(),
            timeout: Cell::new(None),
        }
    }

    pub fn new_complete(
        id: u8,
        checks: FrozenVec<&'s Check<'s>, MAX_CHECKS_PER_STATE>,
        commands: FrozenVec<&'s Command, MAX_COMMANDS_PER_STATE>,
        timeout: Option<Timeout<'s>>,
    ) -> Self {
        Self {
            id,
            checks,
            commands,
            timeout: Cell::new(timeout),
        }
    }

    pub(crate) fn add_check(&self, check: &'s Check<'s>) -> Result<(), &'s Check<'s>> {
        self.checks.push(check)
    }

    pub(crate) fn add_command(&self, command: &'s Command) -> Result<(), &'s Command> {
        self.commands.push(command)
    }

    pub(crate) fn set_timeout(&self, timeout: Option<Timeout<'s>>) {
        self.timeout.set(timeout);
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
