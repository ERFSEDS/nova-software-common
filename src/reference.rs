//! State machine data structures that use references to reference state transitions.
//! This module's types are uses as opposed to [`index`] during runtime, when being able to easily
//! reference a different state is important

use core::cell::Cell;
use heapless::Vec;

use crate::{frozen::FrozenVec, Command, MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES};

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
}

pub struct Check<'s> {
    pub data: crate::CheckData,
    pub transition: Option<StateTransition<'s>>,
}

impl<'s> Check<'s> {
    pub fn new(data: crate::CheckData, transition: Option<StateTransition<'s>>) -> Self {
        Self { data, transition }
    }
}

#[derive(Copy, Clone)]
pub enum StateTransition<'s> {
    Transition(&'s State<'s>),
    Abort(&'s State<'s>),
}
