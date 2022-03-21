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
    for (i, state) in uninit.iter_mut().enumerate() {
        *state = MaybeUninit::new(State::new(i as u8));
    }

    // # SAFETY: All of the slice's MaybeUninit<T> are initialized from the for loop above.
    // Therefore it is safe to call the below code, as per the code's safety requirements

    // TODO: Change to `MaybeUninit::slice_assume_init_ref` once const_maybe_uninit_assume_init is
    // stabilized
    //
    // See: https://github.com/rust-lang/rust/issues/86722
    let init = unsafe {
        // Code is from slice_assume_init_ref's implementation...
        //
        // # SAFETY: casting slice to a `*const [T]` is safe since the caller guarantees that
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
            if ref_state.checks.push(ref_check).is_err() {
                // The size of `index::State::checks` and `reference::State::checks` is determined
                // by the same constant, so it is impossible to for one vector to have more
                // elements than the capacity of the other
                panic!("State checks exceeded maxmimum number of checks allowed");
            }
        }

        for command in state.commands.iter() {
            let ref_command = alloc_struct(*command, alloc).unwrap();
            if ref_state.commands.push(ref_command).is_err() {
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
            let dest_state = ref_states.get::<usize>((*s).into()).unwrap();
            reference::StateTransition::Transition(dest_state)
        }
        index::StateTransition::Abort(s) => {
            let dest_state = ref_states.get::<usize>((*s).into()).unwrap();
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

#[cfg(test)]
mod tests {
    use crate::{
        index::{Check, ConfigFile, State, StateIndex, StateTransition, Timeout},
        indices_to_refs, CheckData, Command, CommandObject, FloatCondition, NativeFlagCondition,
        PyroContinuityCondition, Seconds, MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES,
    };
    use heapless::Vec;
    use static_alloc::Bump;

    const STATE_SIZE: usize = core::mem::size_of::<State>() * MAX_STATES;
    const CHECK_SIZE: usize = core::mem::size_of::<Check>() * MAX_CHECKS_PER_STATE * MAX_STATES;
    const COMMAND_SIZE: usize =
        core::mem::size_of::<Command>() * MAX_COMMANDS_PER_STATE * MAX_STATES;
    const BUMP_SIZE: usize = STATE_SIZE + CHECK_SIZE + COMMAND_SIZE;

    static A: Bump<[u8; BUMP_SIZE]> = Bump::uninit();

    #[test]
    fn test_indices_to_refs() {
        let mut states = Vec::new();

        //
        // [[states]]
        // name = "Safe"
        //
        let safe = State::new(Vec::new(), Vec::new(), None);
        states.push(safe).unwrap();
        // # SAFETY: We just pushed `safe`
        let safe_idx = unsafe { StateIndex::new_unchecked(states.len() as u8 - 1) };

        //
        // [[states]]
        // name = "Descent"
        //
        // [[states.commands]]
        // object = "DataRate"
        // value = 20
        // time = 0.0
        //
        let mut descent_commands = Vec::new();
        descent_commands
            .push(Command::new(CommandObject::DataRate(20), Seconds(0.0)))
            .unwrap();
        let descent = State::new(Vec::new(), descent_commands, None);
        states.push(descent).unwrap();
        // # SAFETY: We just pushed `descent`
        let descent_idx = unsafe { StateIndex::new_unchecked(states.len() as u8 - 1) };

        //
        // [[states]]
        // name = "Flight"
        //
        // [[states.checks]]
        // name = "ApogeeCheck"
        // object = "ApogeeFlag"
        // type = "Flag"
        // value = false
        // transition = "Descent"
        //
        let mut flight_checks = Vec::new();
        flight_checks
            .push(Check::new(
                CheckData::ApogeeFlag(NativeFlagCondition(true)),
                Some(StateTransition::Transition(descent_idx)),
            ))
            .unwrap();
        let flight = State::new(flight_checks, Vec::new(), None);
        states.push(flight).unwrap();
        // # SAFETY: We just pushed `flight`
        let flight_idx = unsafe { StateIndex::new_unchecked(states.len() as u8 - 1) };

        //
        // [[states]]
        // name = "Launch"
        //
        // [[states.checks]]
        // name = "AltitudeCheck"
        // object = "Altitude"
        // type = "FloatCondition"
        // value = "200.0"
        // transition = "Flight"
        //
        let mut launch_checks = Vec::new();
        launch_checks
            .push(Check::new(
                CheckData::Altitude(FloatCondition::GreaterThan(200.0)),
                Some(StateTransition::Transition(flight_idx)),
            ))
            .unwrap();
        let launch = State::new(launch_checks, Vec::new(), None);
        states.push(launch).unwrap();
        // # SAFETY: We just pushed `launch`
        let launch_idx = unsafe { StateIndex::new_unchecked(states.len() as u8 - 1) };

        //
        // [[states]]
        // name = "Poweron"
        //
        // [[states.checks]]
        // name = "Pyro1Check"
        // object = "Pyro1Continuity"
        // type = "PyroContinuityCondition"
        // value = false
        // abort = "Safe"
        //
        // [[states.checks]]
        // name = "Pyro2Check"
        // object = "Pyro2Continuity"
        // type = "PyroContinuityCondition"
        // value = false
        // abort = "Safe"
        //
        // [[states.checks]]
        // name = "Pyro3Check"
        // object = "Pyro3Continuity"
        // type = "PyroContinuityCondition"
        // value = false
        // abort = "Safe"
        //
        let mut poweron_checks = Vec::new();
        poweron_checks
            .push(Check::new(
                CheckData::Pyro1Continuity(PyroContinuityCondition(false)),
                Some(StateTransition::Abort(safe_idx)),
            ))
            .unwrap();
        poweron_checks
            .push(Check::new(
                CheckData::Pyro2Continuity(PyroContinuityCondition(false)),
                Some(StateTransition::Abort(safe_idx)),
            ))
            .unwrap();
        poweron_checks
            .push(Check::new(
                CheckData::Pyro3Continuity(PyroContinuityCondition(false)),
                Some(StateTransition::Abort(safe_idx)),
            ))
            .unwrap();
        let poweron = State::new(
            poweron_checks,
            Vec::new(),
            Some(Timeout::new(1.0, StateTransition::Transition(launch_idx))),
        );
        states.push(poweron).unwrap();
        // # SAFETY: We just pushed `poweron`
        let poweron_idx = unsafe { StateIndex::new_unchecked(states.len() as u8 - 1) };

        let config = ConfigFile {
            default_state: poweron_idx,
            states: states.clone(),
        };

        let reference_cfg = indices_to_refs(&config, &A).unwrap();

        // Test to see if the "reference states" match the "index states" in every way
        for (i, (state, idx_state)) in reference_cfg.iter().zip(states.iter()).enumerate() {
            assert_eq!(state.id, i as u8);
            assert_eq!(state.checks.len(), idx_state.checks.len());
            assert_eq!(state.commands.len(), idx_state.commands.len());

            for (check, idx_check) in state.checks.iter().zip(idx_state.checks.iter()) {
                assert_eq!(check.data, idx_check.data);

                assert_eq!(check.transition.is_some(), idx_check.transition.is_some());

                if let Some(transition) = check.transition {
                    let idx_transition = idx_check.transition.unwrap();

                    match transition {
                        crate::reference::StateTransition::Transition(s) => match idx_transition {
                            crate::index::StateTransition::Transition(idx) => {
                                assert_eq!(s.id, usize::from(idx) as u8);
                            }
                            crate::index::StateTransition::Abort(_) => {
                                panic!();
                            }
                        },
                        crate::reference::StateTransition::Abort(s) => match idx_transition {
                            crate::index::StateTransition::Abort(idx) => {
                                assert_eq!(s.id, usize::from(idx) as u8);
                            }
                            crate::index::StateTransition::Transition(_) => {
                                panic!();
                            }
                        },
                    }
                }
            }

            for (command, idx_command) in state.commands.iter().zip(idx_state.commands.iter()) {
                assert_eq!(command.object, idx_command.object);
                assert_eq!(command.delay, idx_command.delay);
            }
        }
    }
}
