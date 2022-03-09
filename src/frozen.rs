use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use stable_deref_trait::StableDeref;

/// A fixed capacity, heapless `FrozenVec` from the `elsa` crate implementation
///
/// use frozen::FrozenVec;
///
/// let vec = FrozenVec::<u32, 8>::new();
///
/// let x = 2;
/// let y = 4;
///
/// vec.push(&x);
/// vec.push(&y);
///
pub struct FrozenVec<T, const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<T>; N]>,
    len: UnsafeCell<usize>,
}

impl<T: StableDeref, const N: usize> FrozenVec<T, N> {
    const INIT: MaybeUninit<T> = MaybeUninit::uninit();

    /// Constructs a new, empty vector with a fixed capacity of `N`
    pub fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([Self::INIT; N]),
            len: UnsafeCell::new(0),
        }
    }

    /// Appends an `item` to the back of the collection
    ///
    /// Returns back the `item` if the vector is full
    pub fn push(&self, item: T) -> Result<(), T> {
        if self.len() < self.capacity() {
            // SAFETY: We have already performed the bounds check to see if we have exceeded our
            // capacity
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Appends an `item` to the back of the collection, but immediately return a shared reference
    /// to it
    pub fn push_get(&self, item: T) -> Result<&T::Target, T> {
        self.push(item)?;

        // SAFETY: We have just pushed an element and if it failed, it would have already returned.
        // Therefore self.len() - 1 is at least 0
        unsafe { Ok(self.get_unchecked(self.len() - 1)) }
    }

    /// Appends an `item` to the back of the collection
    ///
    /// # SAFETY:
    /// This assumes the vector is not full
    pub unsafe fn push_unchecked(&self, item: T) {
        debug_assert!(!self.is_full());

        let current_len = self.len();

        // SAFETY:
        // 1. This FrozenVec is !Send, so we are the only ones accessing this UnsafeCell.
        // 2. The buffer consists of MaybeUninit<T> instead of raw T, so we can directly write into
        //    the array instead of using ptr::write because MaybeUninit will never drop the inner
        //    T, so we won't be dropping random garbage.
        let buffer = &mut *(self.buffer.get());
        *buffer.get_unchecked_mut(current_len) = MaybeUninit::new(item);

        // SAFETY: We are the only ones currently borrowing len, as it is impossible to do from
        // safe code, and we give up our mutable reference here immediately, as does any other
        // function on Self
        let len = &mut *(self.len.get());
        *len = current_len + 1;
    }

    /// Returns a reference to an element
    pub fn get(&self, index: usize) -> Option<&T::Target> {
        // SAFETY:
        // 1. We never borrow our internal buffer, and instead use raw pointers to index it as a
        //    slice, allowing us to dereference one of our buffers' T: StableDeref's which are
        //    themselves borrowed instead of any part of our buffer
        unsafe {
            let buffer = self.buffer.get();
            (*buffer).get(index).map(|x| x.assume_init_ref().deref())
        }
    }

    /// Returns a reference to an element, without doing bounds checking.
    ///
    /// # SAFETY:
    ///
    /// `index` must be in bounds, i.e. it must be less than `self.len()`
    pub unsafe fn get_unchecked(&self, index: usize) -> &T::Target {
        // SAFETY:
        // 1. We never borrow our internal buffer, and instead use raw pointers to index it as a
        //    slice, allowing us to dereference one of our buffers' T: StableDeref's which are
        //    themselves borrowed instead of any part of our buffer
        // 2. The bounds check was performed prior to this function call
        let buffer = self.buffer.get();
        (*buffer).get_unchecked(index).assume_init_ref().deref()
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

    /// Returns the current length of the vector (the number of elements currently stored in it)
    ///
    /// NOTE: This is not the capacity of the vector, which is the maximum number of elements that
    /// can be stored.
    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: Here we completely bypass creating a reference and only read the value from
        // self.len. Therefore this will always be valid
        unsafe { *self.len.get() }
    }

    /// Returns the maximum number of elements the vector can hold
    pub fn capacity(&self) -> usize {
        N
    }
}

/// Iterator over FrozenVec, obtained via `.iter()`
///
/// It is safe to push to the vector during iteration
pub struct Iter<'a, T, const N: usize> {
    vec: &'a FrozenVec<T, N>,
    idx: usize,
}

impl<'a, T: StableDeref, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T::Target;
    fn next(&mut self) -> Option<&'a T::Target> {
        if let Some(ret) = self.vec.get(self.idx) {
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl<'a, T: StableDeref, const N: usize> IntoIterator for &'a FrozenVec<T, N> {
    type Item = &'a T::Target;
    type IntoIter = Iter<'a, T, N>;
    fn into_iter(self) -> Iter<'a, T, N> {
        Iter { vec: self, idx: 0 }
    }
}

impl<T: StableDeref, const N: usize> Default for FrozenVec<T, N> {
    fn default() -> Self {
        Self::new()
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
