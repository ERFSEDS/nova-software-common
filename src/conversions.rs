use crate::index::StateIndex;
use crate::{index, reference, Check, State, StateTransition, Timeout, MAX_STATES};

use alloc::alloc;
use alloc_traits::{Layout, LocalAlloc, NonZeroLayout};
use core::mem::{align_of, size_of, MaybeUninit};

/// Allocates a slice that is `size` elements long and fills it with data from `next`.
///
/// Returns `None` if `next` yields less than`size` elements, or if the allocation fails
fn alloc_slice_uninit<T>(
    size: usize,
    mut next: impl Iterator<Item = T>,
    alloc: &'static dyn LocalAlloc<'static>,
) -> Option<&'static mut [MaybeUninit<T>]> {
    let bytes = size * size_of::<T>();
    let align = align_of::<T>();

    // Unwrap always succeeds because align was obtained from `align_of`
    let layout: Layout = alloc::Layout::from_size_align(bytes, align).unwrap().into();
    let layout = NonZeroLayout::from_layout(layout).unwrap();
    let mem = alloc.alloc(layout)?;

    // # SAFETY
    // 1. `mem` is a valid, aligned, non-null pointer
    // 2. `mem` was obtained from a single allocation via [`LocalAlloc::alloc`]
    // 3. `mem` is safe for reads up to `bytes` bytes because of the layout passed to `alloc`
    let uninit: &'static mut [MaybeUninit<T>] =
        unsafe { core::slice::from_raw_parts_mut(mem.ptr.as_ptr() as *mut MaybeUninit<T>, size) };

    for dst_state in uninit.iter_mut() {
        dst_state.write(next.next()?);
    }
    Some(uninit)
}

/// Allocates a slice that is `size` elements long and fills it with data from `next`.
///
/// Returns `None` if `next` yields less than`size` elements, or if the allocation fails
fn alloc_slice<T>(
    size: usize,
    next: impl Iterator<Item = T>,
    alloc: &'static dyn LocalAlloc<'static>,
) -> Option<&'static mut [T]> {
    let uninit = alloc_slice_uninit(size, next, alloc)?;

    // # SAFETY
    // 1. All elements of `uninit` have been initialized

    // TODO: Change to `MaybeUninit::slice_assume_init_ref` once const_maybe_uninit_assume_init is
    // stabilized
    //
    // See: https://github.com/rust-lang/rust/issues/86722
    let result: &'static mut [T] = unsafe {
        // Code is from slice_assume_init_ref's implementation...
        //
        // SAFETY: casting slice to a `*const [T]` is safe since the caller guarantees that
        // `slice` is initialized, and`MaybeUninit` is guaranteed to have the same layout as `T`.
        // The pointer obtained is valid since it refers to memory owned by `slice` which is a
        // reference and thus guaranteed to be valid for reads.
        &mut *(uninit as *mut [MaybeUninit<T>] as *mut [T])
    };
    Some(result)
}

/// Allocates a slice that is `size` elements long and fills it with data from `next`.
///
/// Returns `None` if `next` yields less than`size` elements, or if the allocation fails
///
/// # Panics
/// Panics if T is a zero sized type
fn alloc_struct<T>(obj: T, alloc: &'static dyn LocalAlloc<'static>) -> Option<&'static mut T> {
    let layout = NonZeroLayout::from_layout(alloc_traits::Layout::new::<T>()).unwrap();
    let mem = alloc.alloc(layout)?;
    let ptr: *mut T = mem.ptr.as_ptr() as *mut T;

    // # SAFETY:
    // `ptr` is a valid, aligned, non-null pointer obtianed from `alloc`
    // `ptr` was uninitalized before
    unsafe { ptr.write(obj) };

    // # SAFETY:
    // `ptr` is a valid pointer with a 'static lifetime obtained from `alloc`
    Some(unsafe { &mut *ptr })
}

/// Converts a serialized config file to a state graph suitable for executing with a state machine.
/// alloc is used to allocate the memory for the returned slice
pub fn indices_to_refs(
    config: &index::ConfigFile,
    alloc: &'static dyn LocalAlloc<'static>,
) -> Option<&'static [reference::State]> {
    use heapless::Vec;
    let states: &'static mut [reference::State] = alloc_slice(
        config.states.len(),
        config
            .states
            .iter()
            .enumerate()
            // `config.states` is limited to `MAX_STATES` which is less than u8::MAX so using as is ok
            .map(|(i, _)| crate::State::new(i as u8, Vec::new(), Vec::new(), None)),
        alloc,
    )?;

    for (state, dst_state) in config.states.iter().zip(states.iter_mut()) {
        for check in &state.checks {
            let transition = match check.transition {
                StateTransition::Abort(s) => StateTransition::Abort(&states[s.as_index()]),
                StateTransition::Transition(s) => {
                    StateTransition::Transition(&states[s.as_index()])
                }
            };
            let dst_check = Check::new(check.object, check.condition, transition);
            let dst_check: &'static Check<reference::StateRef>  = alloc_struct(dst_check, alloc)?;
            dst_state
                .checks
                .push(dst_check)
                .unwrap_or_else(|_| unreachable!());
        }
        for command in &state.commands {}
    }

    Some(states)
}

/// Returns the index of `state` inside `states` if present.
/// Returns None if `state` was not found in `slice`
pub fn get_state_index(
    states: &[reference::StateRef],
    state: reference::StateRef,
) -> Option<StateIndex> {
    let position = states.iter().position(|&s| s == state);
    // SAFETY: `state` was found in `slice` at index `i`, so it is a valid index
    position.map(|i| unsafe { StateIndex::new_unchecked(i) })
}

fn transition_ref_to_index(
    transition: StateTransition<reference::StateRef>,
    states: &[reference::StateRef],
) -> StateTransition<index::StateIndex> {
    match transition {
        StateTransition::Abort(state) => {
            let index = get_state_index(states, state).unwrap();
            StateTransition::Abort(index)
        }
        StateTransition::Transition(state) => {
            let index = get_state_index(states, state).unwrap();
            StateTransition::Abort(index)
        }
    }
}

pub fn refs_to_indices(config: &reference::ConfigFile) -> index::ConfigFile {
    use heapless::Vec;

    let mut states: Vec<_, MAX_STATES> = config
        .states
        .iter()
        .enumerate()
        .map(|(i, _)| index::State::new(i as u8, Vec::new(), Vec::new(), None))
        .collect();

    for (dst_state, src_state) in states.iter_mut().zip(config.states.iter()) {
        dst_state.timeout = src_state.timeout.as_ref().map(|src_timeout| {
            let transition = transition_ref_to_index(src_timeout.transition, &config.states);
            Timeout::new(src_timeout.time, transition)
        });

        for src_check in &src_state.checks {
            let transition = transition_ref_to_index(src_check.transition, &config.states);
            let check = Check {
                object: src_check.object,
                condition: src_check.condition,
                transition,
            };
            // `states.checks` has the same capacity as `dst_state.checks`
            dst_state
                .checks
                .push(check)
                .unwrap_or_else(|_| unreachable!());
        }

        for &src_command in src_state.commands.iter() {
            dst_state.commands.push((src_command).into()).unwrap();
        }
    }

    // unwrap: default state is guarnteed to be in config.states
    let default_state = get_state_index(config.states.as_slice(), config.default_state).unwrap();
    index::ConfigFile {
        default_state,
        states,
    }
}
