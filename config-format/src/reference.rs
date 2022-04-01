//! State machine data structures that use references to reference state transitions.
//! This module's types are uses as opposed to [`index`] during runtime, when being able to easily
//! reference a different state is important

use core::cell::Cell;
use core::option::Option;
use heapless::Vec;

use super::{frozen::FrozenVec, Seconds, MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES};

pub struct ConfigFile<'s> {
    pub default_state: &'s State<'s>,
    pub states: Vec<&'s State<'s>, MAX_STATES>,
}

#[derive(Copy, Clone)]
pub struct Timeout<'s> {
    pub time: Seconds,
    pub transition: StateTransition<'s>,
}

impl<'s> Timeout<'s> {
    pub fn new(time: Seconds, transition: StateTransition<'s>) -> Self {
        Self { time, transition }
    }
}

#[derive(Clone)]
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
}

#[derive(Copy, Clone)]
pub struct Check<'s> {
    pub data: super::CheckData,
    pub transition: Option<StateTransition<'s>>,
}

impl<'s> Check<'s> {
    pub fn new(data: super::CheckData, transition: Option<StateTransition<'s>>) -> Self {
        Self { data, transition }
    }
}

#[derive(Copy, Clone)]
pub enum StateTransition<'s> {
    Transition(&'s State<'s>),
    Abort(&'s State<'s>),
}

/// An action that takes place at a specific time after the state containing this is entered
#[derive(Debug)]
pub struct Command {
    /// The object that this command will act upon
    pub object: super::CommandValue,

    /// How long after the state activates to execute this command
    pub delay: super::Seconds,

    /// If this command has already executed
    pub was_executed: Cell<bool>,
}

impl Command {
    pub fn new(object: super::CommandValue, delay: super::Seconds) -> Self {
        Self {
            object,
            delay,
            was_executed: Cell::new(false),
        }
    }
}
