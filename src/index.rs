//! State machine data structures that use indices to reference state transitions.
//! This is needed when the config file is serialized between the verifier and the flight computer.

use crate::{Check, Timeout};

use serde::{Deserialize, Serialize};

pub type State = crate::State<Check<StateIndex>, Timeout<StateIndex>>;
pub type StateRef = StateIndex;
pub type ConfigFile = crate::ConfigFile<State, StateIndex>;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
/// The which references a particural state
pub struct StateIndex(u8);

impl StateIndex {
    /// Creates a new `StateIndex` without checking that `index` is valid. The validity of `index`
    /// is based on the context in which it is used, therefore we provide no safe new function. The
    /// caller must take the responsibility that this new `StateIndex` makes senes for its own use case
    ///
    /// # Safety
    /// The caller must guarntee that index is valid within the context of the value it is
    /// referencing
    ///
    /// Note: Passing in an invalid index here will never lead to memory unsafety.
    /// This wrapper simply allows us to feel better about unwrapping `get()`s that use index at
    /// other places in the codebase because we assume constructing an invalid `StateIndex` is
    /// impossible
    pub unsafe fn new_unchecked(index: usize) -> Self {
        StateIndex(index as u8)
    }

    pub fn as_index(self) -> usize {
        self.0 as usize
    }
}

impl From<StateIndex> for usize {
    fn from(index: StateIndex) -> Self {
        index.as_index()
    }
}

#[test]
fn test() {
    assert_eq!(core::mem::size_of::<ConfigFile>(), 0);
}
