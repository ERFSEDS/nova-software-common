use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

/// A fixed capacity, heapless `FrozenVec` based on the `elsa` crate's implementation.
///
/// Called frozen because items can never be removed.
///
/// ```
/// use frozen::FrozenVec;
///
/// let vec = FrozenVec::<u32, 8>::new();
///
/// // Notice we have shared reference
/// a(&vec);
/// fn a(vec: &FrozenVec::<u32, 8>) {
///     vec.push(2);
///     vec.push(5);
///     vec.push(10);
///
///     //Items can never be removed, which is how we can safely do this with a shared reference!
///     assert_eq!(vec[0], 2);
///     assert_eq!(vec.get(1).unwrap(), 5);
///     assert_eq!(vec.iter().last().unwrap(), 10);
/// }
/// // Because all operations can be done using shared references, `FrozenVec` is `!Sync` to
/// // enforce aliasing concerns
/// ```
// Internally each element is an UnsafeCell<MaybeUninit<T>>. We need the inner MaybeUninit<>
// because unoccupied entries are uninitalized for performance reasons. We keep track of how many
// elements are initilased with `len`
//
// The reason this whole construct is safe with only shared references, is because the only time
// the frozen vec is mutated is when a new element is pushed. When this happens, we obtain a
// mutable refernce to `len` in order to increment it, as well as a mutable reference to the
// UnsafeCell at offset `i` in order to move the to-be-pushed value into it. Because FrozenVec is
// `!Sync`, this can never happen from mutiple threads at the same time, therefore the mutable
// references that alias `len` and `buf[i]` are safe.
//
// Because items cannot be removed, the only time `buf[i]` will be mutated is when an item is
// pushed there, which is guarnteed to be _before_ a user could call `get(i)` and get a reference
// to it, because before the call to `push`, `get(i)` would return `None`, as `i` is out of bounds.
//
// Additionally we never give out mutable references to the elements inside `FrozenVec`, otherwise
// a user could spam `get_mut(i)` and invoke UB.
pub struct FrozenVec<T, const N: usize> {
    /// The elements within this frozen vec.
    ///
    /// Only elements 0..len are initalized
    buf: [UnsafeCell<MaybeUninit<T>>; N],
    len: UnsafeCell<usize>,
}

impl<T, const N: usize> FrozenVec<T, N> {
    /// Constructs a new, empty vector with a fixed capacity of `N`
    pub fn new() -> Self {
        Self {
            buf: [UnsafeCell::new(MaybeUninit::uninit()); N],
            len: UnsafeCell::new(0),
        }
    }

    /// Appends `item` to the back of the collection
    ///
    /// Returns back `item` in Err(..) if the vector is full
    pub fn push(&self, item: T) -> Result<(), T> {
        if self.is_full() {
            Err(item)
        } else {
            // # SAFETY: we have space for another element by the bounds check above
            Ok(unsafe { self.push_unchecked(item) })
        }
    }

    /// Appends `item` to the back of the collection, immediately returning a shared reference
    /// to it.
    ///
    /// Returns back `item` in Err(..) if the vector is full
    fn push_get(&self, item: T) -> Result<&T, T> {
        if self.is_full() {
            Err(item)
        } else {
            // # SAFETY: we have space for another element by the bounds check above
            Ok(unsafe { self.push_get_unchecked(item) })
        }
    }

    /// Appends `item` to the back of the collection and returns a reference to it
    ///
    /// # Safety
    ///
    /// This assumes the vector is not full
    pub unsafe fn push_get_unchecked(&self, item: T) -> &T {
        // # SAFETY: `Self` is !Send, so we are the only ones accessing `self.len`
        let len = &mut *self.len.get();

        // The index of the to-be-pushed element is the current size of this FrozenVec
        let i: usize = *len;
        *len += 1;
        // TODO: in the future we might be able to make this FrozenVec Sync by making `len` atomic.
        // The code is written like this to show the fetch-increment used to obtain a new unique index

        // # SAFETY:
        // 1. `Self` is !Send, so the increment of `i` creates a new unique index for this `FrozenVec`
        // 2. Because `i` is unique are the only ones accessing the UnsafeCell at index `i`
        // (shared references may exist to other elements, but they are inside other UnsafeCells)
        let elem = unsafe { &mut *self.buf[i].get() };

        // `elem` is uninitalized before because i is unique, so we don't overwrite anything here
        elem.write(item)
    }

    /// Appends an `item` to the back of the collection and returns a reference to it
    ///
    /// # Safety
    ///
    /// This assumes the vector is not full
    pub unsafe fn push_unchecked(&self, item: T) {
        // # SAFETY: The caller has guarnteed that the vector is not full
        unsafe { self.push_get_unchecked(item) };
    }

    /// Returns a reference to an element
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            // # SAFETY: `index` is in bounds by the bounds check above
            Some(unsafe { self.get_unchecked(index) })
        } else {
            None
        }
    }

    /// Returns a reference to an element, without doing bounds checking.
    ///
    /// # Safety
    ///
    /// `index` must be in bounds, i.e. it must be less than `self.len()`
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        // # SAFETY:
        // The user has guarnteed that `index` is in bounds
        let elem = unsafe { self.buf.get_unchecked(index).get() };

        // # SAFETY:
        // 1. `index` is in bounds, therefore `elem` is initialized
        // 2. `elem` is only mutably aliased during push operations, which happened already since
        //    `elem` is initialized
        unsafe { (*elem).assume_init_ref() }
    }

    /// Returns an iterator over the vector.
    pub fn iter(&self) -> Iter<T, N> {
        self.into_iter()
    }

    /// Returns true if the vector is full
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Returns true if the vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the current length of the vector (the number of elements currently stored in it).
    /// This is always equal to the number of push operations that have been performed.
    ///
    /// NOTE: This is not the capacity of the vector, which is the maximum number of elements that
    /// can be stored.
    #[inline]
    pub fn len(&self) -> usize {
        // # SAFETY: `Self` is !Sync, and this function is not called by any other function that
        // mutibly aliases `self.len`
        unsafe { *self.len.get() }
    }

    /// Returns the maximum number of elements the vector can hold
    pub fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Clone for FrozenVec<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let len = self.len();
        const UNINIT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

        let dst = [UNINIT; N];
        for (i, val) in self.iter().enumerate() {
            dst[i] = UnsafeCell::new(MaybeUninit::new(val.clone()));
        }
        // We have initialized len elements of `dst`
        Self {
            buf: dst,
            len: UnsafeCell::new(len),
        }
    }
}

/// Iterator over FrozenVec, obtained via `.iter()`
///
/// It is safe to push to the vector during iteration
pub struct Iter<'a, T, const N: usize> {
    vec: &'a FrozenVec<T, N>,
    idx: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if let Some(ret) = self.vec.get(self.idx) {
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a FrozenVec<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;
    fn into_iter(self) -> Iter<'a, T, N> {
        Iter { vec: self, idx: 0 }
    }
}

impl<T, const N: usize> Default for FrozenVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> core::ops::Index<usize> for FrozenVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("index out of range {index}, cap {}", self.capacity()))
    }
}

#[test]
fn test_iteration() {
    use heapless::Vec;

    let v: FrozenVec<&u32, 6> = FrozenVec::new();
    let mut h: Vec<&u32, 6> = Vec::new();

    let x = 0;
    let y = 2;
    let z = 4;
    let a = 7;
    let b = 9;
    let c = 11;

    v.push(&x).unwrap();
    v.push(&y).unwrap();
    v.push(&z).unwrap();
    v.push(&a).unwrap();
    v.push(&b).unwrap();
    v.push(&c).unwrap();

    h.push(&x).unwrap();
    h.push(&y).unwrap();
    h.push(&z).unwrap();
    h.push(&a).unwrap();
    h.push(&b).unwrap();
    h.push(&c).unwrap();

    for (e1, e2) in h.iter().zip(v.iter()) {
        assert_eq!(*e1, e2);
    }

    assert_eq!(h.len(), v.iter().count());

    // We need to drop this first
    drop(h);
}

#[test]
fn test_accessors() {
    let vec: FrozenVec<&u32, 8> = FrozenVec::new();

    let x = 0;
    let y = 2;
    let z = 4;

    assert_eq!(vec.is_empty(), true);
    assert_eq!(vec.len(), 0);
    // assert_eq!(vec.first(), None);
    // assert_eq!(vec.last(), None);
    assert_eq!(vec.get(1), None);

    vec.push(&x).unwrap();
    vec.push(&y).unwrap();
    vec.push(&z).unwrap();

    assert_eq!(vec.is_empty(), false);
    assert_eq!(vec.len(), 3);
    // assert_eq!(vec.first(), Some("a"));
    // assert_eq!(vec.last(), Some("c"));
    assert_eq!(vec.get(1), Some(&y));
}
