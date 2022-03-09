//! State machine data structures that use indices to reference state transitions.
//! This is needed when the config file is serialized between the verifier and the flight computer.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
/// An index into a particural `Vec` inside [`ConfigFile`]
pub struct Index(u8);

impl Index {
    /// Creates a new `StateIndex` without checking that `index` is valid. The validity of `index`
    /// is based on the context in which it is used, therefore we provide no safe new function. The
    /// caller must take the responsibility that this new `StateIndex` makes senes for its own use case
    ///
    /// # Safety
    /// The caller must guarntee that index is valid within the context of the value it is
    /// referencing. Reads into Vec's will assume this index is in bounds and use [`get_unchecked`]
    /// for performanace reasons
    pub unsafe fn new_unchecked(index: u8) -> Self {
        Self(index)
    }
}

impl From<Index> for usize {
    fn from(index: Index) -> Self {
        index.0 as usize
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct StateIndex(Index);

impl StateIndex {
    /// See [`Index::new_unchecked`]
    ///
    /// # Safety
    /// The caller must guarntee that index is within bounds for the state in the states array that
    /// it refers to
    pub unsafe fn new_unchecked(index: u8) -> Self {
        // SAFETY: Caller taks responsibility for the contract of `Index::new_unchecked`
        unsafe { Self(Index::new_unchecked(index)) }
    }
}

impl From<StateIndex> for usize {
    fn from(index: StateIndex) -> Self {
        index.0.into()
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct CheckIndex(Index);

impl CheckIndex {
    /// See [`Index::new_unchecked`]
    ///
    /// # Safety
    /// The caller must guarntee that index is within bounds for the command in the commands array that
    /// it refers to
    pub unsafe fn new_unchecked(index: u8) -> Self {
        // SAFETY: Caller taks responsibility for the contract of `Index::new_unchecked`
        unsafe { Self(Index::new_unchecked(index)) }
    }
}

impl From<CheckIndex> for usize {
    fn from(index: CheckIndex) -> Self {
        index.0.into()
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct CommandIndex(Index);

impl CommandIndex {
    /// See [`Index::new_unchecked`]
    ///
    /// # Safety
    /// The caller must guarntee that index is within bounds for the command in the commands array that
    /// it refers to
    pub unsafe fn new_unchecked(index: u8) -> Self {
        // SAFETY: Caller taks responsibility for the contract of `Index::new_unchecked`
        unsafe { Self(Index::new_unchecked(index)) }
    }
}

impl From<CommandIndex> for usize {
    fn from(index: CommandIndex) -> Self {
        index.0.into()
    }
}
