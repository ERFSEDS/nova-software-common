use crate::{index, reference};
use alloc::alloc;
use alloc_traits::{Layout, LocalAlloc, NonZeroLayout};
use core::mem::{align_of, size_of, MaybeUninit};
use core::slice;

type State = reference::State<'static>;

/// Converts a serialized config file to a state graph suitable for executing with a state machine.
/// alloc is used to allocate the memory for the returned slice
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
    let uninit: &'static [MaybeUninit<State>] =
        unsafe { slice::from_raw_parts(mem.ptr.as_ptr() as *const _, len) };

    // # SAFETY
    // 1. The non-reference values in `uninit` have been initialized
    // 2. The reference values in `uninit` have been initialized

    // TODO: Change to `MaybeUninit::slice_assume_init_ref` once const_maybe_uninit_assume_init is
    // stabilized
    //
    // See: https://github.com/rust-lang/rust/issues/86722
    let result = unsafe {
        // Code is from slice_assume_init_ref's implementation...
        //
        // SAFETY: casting slice to a `*const [T]` is safe since the caller guarantees that
        // `slice` is initialized, and`MaybeUninit` is guaranteed to have the same layout as `T`.
        // The pointer obtained is valid since it refers to memory owned by `slice` which is a
        // reference and thus guaranteed to be valid for reads.
        &*(uninit as *const [MaybeUninit<State>] as *const [State])
    };
    Some(result)
}

pub fn refs_to_indices(config: &reference::ConfigFile) -> index::ConfigFile {
    todo!()
}
