//! State machine data structures that use references to reference state transitions.
//! This module's types are uses as opposed to [`index`] during runtime, when being able to easily
//! reference a different state is important
use core::marker::PhantomData;

use crate::{Check, Timeout};

pub type State = crate::State<&'static Check<StateRef>, &'static Timeout<StateRef>>;
pub type ConfigFile<'s> = crate::ConfigFile<StateRef, StateRef>;

/// A refrence to a &'static State. Used for breaking cycles in generic types
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StateRef {
    ptr: *const State,
    phantom_: PhantomData<&'static State>,
}

impl StateRef {
    /// Creates a state reference based
    pub fn new<Check, Timeout>(
        val: &'static State,
    ) -> Self {
        Self {
            ptr: val as *const _,
            phantom_: PhantomData,
        }
    }
}

impl core::ops::Deref for StateRef {
    type Target = crate::State<crate::Check<StateRef>, crate::Timeout<StateRef>>;

    fn deref(&self) -> &Self::Target {
        // FIXME: This is UB if called while we have mutable references to states
        &unsafe { *self.ptr }
    }
}
