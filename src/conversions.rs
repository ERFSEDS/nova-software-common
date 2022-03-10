use crate::reference::Check;
use crate::{index, reference};

use alloc::alloc;
use alloc_traits::{Layout, LocalAlloc, NonZeroLayout};
use core::mem::{align_of, size_of, MaybeUninit};
use core::slice;

type State = reference::State<'static>;

pub fn indices_to_refs(
    config: &index::ConfigFile,
    alloc: &'static dyn LocalAlloc<'static>,
) -> Option<&'static [State]> {
    let len = config.states.len();
    let bytes = len * size_of::<State>();
    let align = align_of::<State>();

    // Unwrap always succeeds because align was obtained from `align_of`
    let layout: Layout = alloc::Layout::from_size_align(bytes, align).unwrap().into();
    let layout = NonZeroLayout::from_layout(layout).unwrap();
    let mem = alloc.alloc(layout)?;

    // # SAFETY
    // 1. `mem` is a valid, aligned, non-null pointer
    // 2. `mem` was obtained from a single allocation via [`LocalAlloc::alloc`]
    // 3. `mem` is safe for reads up to `bytes` bytes
    // 4. `mem` is only being accessed through this slice, and therefore this mutable reference is
    //    not aliased
    let uninit: &'static mut [MaybeUninit<State>] =
        unsafe { slice::from_raw_parts_mut(mem.ptr.as_ptr() as *mut _, len) };

    // Create a new, initialized State at each position in the slice
    for i in 0..config.states.len() {
        uninit[i] = MaybeUninit::new(State::new(i as u8));
    }

    // FIXME: actually initialize

    // TODO: Change to `MaybeUninit::slice_assume_init_ref` once const_maybe_uninit_assume_init is
    // stabilized
    //
    // See: https://github.com/rust-lang/rust/issues/86722
    let init = unsafe {
        // Code is from slice_assume_init_ref's implementation...
        //
        // SAFETY: casting slice to a `*mut [T]` is safe since the caller guarantees that
        // `slice` is initialized, and`MaybeUninit` is guaranteed to have the same layout as `T`.
        // The pointer obtained is valid since it refers to memory owned by `uninit` which is a
        // reference and thus guaranteed to be valid for reads.
        &*(uninit as *const [MaybeUninit<State>] as *const [State])
    };

    // Now that each state is initialized, we can add the proper checks, commands, and timeouts
    for (i, state) in config.states.iter().enumerate() {
        let ref_state = init.get(i).unwrap();

        for check in state.checks.iter() {
            let transition = check
                .transition
                .as_ref()
                .map(|t| transition_index_to_ref(t, init));

            // Create and add the check
            let ref_check = Check::new(check.data, transition);
            let ref_check = alloc_struct(ref_check, alloc).unwrap();
            if let Err(_) = ref_state.checks.push(ref_check) {
                // The size of `index::State::checks` and `reference::State::checks` is determined
                // by the same constant, so it is impossible to for one vector to have more
                // elements than the capacity of the other
                panic!("State checks exceeded maxmimum number of checks allowed");
            }
        }

        for command in state.commands.iter() {
            let ref_command = alloc_struct(command.into(), alloc).unwrap();
            if let Err(_) = ref_state.commands.push(ref_command) {
                // The size of `index::State::commands` and `reference::State::commands` is determined
                // by the same constant, so it is impossible to for one vector to have more
                // elements than the capacity of the other
                panic!("State commands exceeded maxmimum number of commands allowed");
            }
        }

        if let Some(timeout) = &state.timeout {
            let timeout_transition = transition_index_to_ref(&timeout.transition, init);
            let ref_timeout = Some(reference::Timeout::new(timeout.time, timeout_transition));
            ref_state.timeout.set(ref_timeout);
        }
    }

    Some(init)
}

fn transition_index_to_ref<'s>(
    transition: &index::StateTransition,
    ref_states: &'s [reference::State<'s>],
) -> reference::StateTransition<'s> {
    match transition {
        index::StateTransition::Transition(s) => {
            let dest_state = ref_states.get::<usize>(s.clone().into()).unwrap();
            reference::StateTransition::Transition(dest_state)
        }
        index::StateTransition::Abort(s) => {
            let dest_state = ref_states.get::<usize>(s.clone().into()).unwrap();
            reference::StateTransition::Abort(dest_state)
        }
    }
}

fn alloc_struct<T>(obj: T, alloc: &'static dyn LocalAlloc<'static>) -> Option<&'static T> {
    let layout = NonZeroLayout::from_layout(alloc_traits::Layout::new::<T>()).unwrap();
    let mem = alloc.alloc(layout)?;
    let ptr: *mut T = mem.ptr.as_ptr() as *mut T;

    // # SAFETY:
    // `ptr` is a valid, aligned, non-null pointer obtianed from `alloc`
    // `ptr` was uninitalized before
    unsafe { ptr.write(obj) };

    // # SAFETY:
    // `ptr` is a valid pointer with a 'static lifetime obtained from `alloc`
    Some(unsafe { &*ptr })
}

pub fn refs_to_indices(config: &reference::ConfigFile) -> index::ConfigFile {
    /*
    use heapless::Vec;
    let mut states: Vec<_, MAX_STATES> = config
        .states
        .iter()
        .map(|a| index::State::new(Vec::new(), Vec::new(), None))
        .collect();

    for (dst_state, src_state) in states.iter_mut().zip(config.states.iter().copied()) {
        dst_state.timeout = src_state.timeout.as_ref().map(|src_timeout| {
            let transition = transition_ref_to_index(&src_timeout.transition, &config.states);
            index::Timeout::new(src_timeout.time, transition)
        });

        for src_check in src_state.checks {
            let transition = transition_ref_to_index(&src_check.transition, &config.states);
            let check = index::Check {
                object: src_check.object,
                condition: src_check.condition,
                transition,
            };
            dst_state.checks.push(check).unwrap();
        }

        for src_command in src_state.commands.iter() {
            dst_state.commands.push((*src_command).into());
        }
    }
    */

    todo!()
}
