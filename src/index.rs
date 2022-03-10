//! State machine data structures that use indices to reference state transitions.
//! This is needed when the config file is serialized between the verifier and the flight computer.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
/// An index into a particural `Vec` inside a [`ConfigFile`]
pub struct Index(pub(crate) u8);

impl Index {
    /// Creates a new `StateIndex` without checking that `index` is valid. The validity of `index`
    /// is based on the context in which it is used, therefore we provide no safe new function. The
    /// caller must take the responsibility that this new `StateIndex` makes senes for its own use case
    ///
    /// # Safety
    /// The caller must guarntee that index is valid within the context of the value it is
    /// referencing. Use of an invalid index in a context that would lead to an out of bounds
    /// access will cause the rocket to panic and fail. 
    ///
    /// Use of an incorrect index here will never lead to memory unsafety, however this method is
    /// still marked unsafe as it will cause a catastrophic failure of the rocket if the caller is
    /// carefree about calling this function
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
pub struct StateIndex(pub(crate) Index);

impl StateIndex {
    /// Creates a new `StateIndex` without checking its validity
    /// # Safety
    /// See [`Index::new_unchecked`]
    pub unsafe fn new_unchecked(index: u8) -> Self {
        // SAFETY: Caller taks responsibility for the contract of `Index::new_unchecked`
        unsafe { Self(Index::new_unchecked(index)) }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct CheckIndex(pub(crate) Index);

impl CheckIndex {
    /// Creates a new `StateIndex` without checking its validity
    /// # Safety
    /// See [`Index::new_unchecked`] 
    pub unsafe fn new_unchecked(index: u8) -> Self {
        // SAFETY: Caller takes responsibility for the contract of `Index::new_unchecked`
        unsafe { Self(Index::new_unchecked(index)) }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct CommandIndex(pub(crate) Index);

impl CommandIndex {
    /// Creates a new `StateIndex` without checking its validity
    /// # Safety
    /// See [`Index::new_unchecked`] 
    pub unsafe fn new_unchecked(index: u8) -> Self {
        // SAFETY: Caller taks responsibility for the contract of `Index::new_unchecked`
        unsafe { Self(Index::new_unchecked(index)) }
    }
}
