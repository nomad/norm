//! An inlineable vector implementation, similar to those of [`arrayvec`] or
//! [`tinyvec`].
//!
//! Those crates do a lot more than we need here though, so we implement our
//! own tiny version.
//!
//! [`arrayvec`]: https://crates.io/crates/arrayvec
//! [`tinyvec`]: https://crates.io/crates/tinyvec

use core::cmp::Ordering;
use core::mem::{transmute, MaybeUninit};
use core::ops::{Index, IndexMut};
use core::ptr;
use core::slice::SliceIndex;

/// TODO: docs
#[derive(Default)]
pub(crate) struct TinyVec<const INLINE: usize, T> {
    inner: TinyVecType<INLINE, T>,
}

impl<const INLINE: usize, T: core::fmt::Debug> core::fmt::Debug
    for TinyVec<INLINE, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.inner {
            TinyVecType::Inline(inner) => inner.fmt(f),
            TinyVecType::Heap(inner) => inner.fmt(f),
        }
    }
}

impl<const INLINE: usize, T, I: SliceIndex<[T]>> Index<I>
    for TinyVec<INLINE, T>
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<const INLINE: usize, T, I: SliceIndex<[T]>> IndexMut<I>
    for TinyVec<INLINE, T>
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<const INLINE: usize, T> TinyVec<INLINE, T> {
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        match &self.inner {
            TinyVecType::Inline(inner) => inner.as_slice(),
            TinyVecType::Heap(inner) => inner.as_slice(),
        }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match &mut self.inner {
            TinyVecType::Inline(inner) => inner.as_mut_slice(),
            TinyVecType::Heap(inner) => inner.as_mut_slice(),
        }
    }

    #[inline(always)]
    pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a T) -> Ordering,
    {
        self.as_slice().binary_search_by(f)
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        match &mut self.inner {
            TinyVecType::Inline(inner) => inner.clear(),
            TinyVecType::Heap(inner) => inner.clear(),
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.as_mut_slice().get_mut(idx)
    }

    #[inline(always)]
    pub fn insert(&mut self, idx: usize, item: T) {
        match &mut self.inner {
            TinyVecType::Inline(inner) => {
                if inner.is_full() {
                    let mut vec = core::mem::take(inner).into_vec();
                    vec.insert(idx, item);
                    self.inner = TinyVecType::Heap(vec);
                } else {
                    inner.insert(idx, item)
                }
            },
            TinyVecType::Heap(inner) => inner.insert(idx, item),
        }
    }

    #[cfg(test)]
    fn is_inline(&self) -> bool {
        matches!(self.inner, TinyVecType::Inline(_))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.inner {
            TinyVecType::Inline(inner) => inner.len(),
            TinyVecType::Heap(inner) => inner.len(),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.insert(self.len(), item);
    }

    #[inline(always)]
    pub fn remove(&mut self, idx: usize) {
        match &mut self.inner {
            TinyVecType::Inline(inner) => inner.remove(idx),
            TinyVecType::Heap(inner) => {
                inner.remove(idx);
            },
        }
    }

    #[inline(always)]
    pub fn split_at_mut(&mut self, mid: usize) -> (&mut [T], &mut [T]) {
        match &mut self.inner {
            TinyVecType::Inline(inner) => {
                inner.as_mut_slice().split_at_mut(mid)
            },
            TinyVecType::Heap(inner) => inner.split_at_mut(mid),
        }
    }
}

/// TODO: docs
enum TinyVecType<const INLINE: usize, T> {
    Inline(InlineVec<INLINE, T>),

    Heap(Vec<T>),
}

impl<const INLINE: usize, T> Default for TinyVecType<INLINE, T> {
    #[inline(always)]
    fn default() -> Self {
        Self::Inline(InlineVec::default())
    }
}

struct InlineVec<const N: usize, T: Sized> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<const N: usize, T: core::fmt::Debug> core::fmt::Debug
    for InlineVec<N, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<const N: usize, T> Default for InlineVec<N, T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is valid.
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<const N: usize, T: Sized> InlineVec<N, T> {
    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        // SAFETY: `MaybeUninit` is layout-transparent and the first `self.len`
        // elements are initialized.
        unsafe { transmute(&self.data[..self.len()]) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: same as `as_slice`.
        unsafe { transmute(&mut self.data[..self.len]) }
    }

    #[inline(always)]
    fn clear(&mut self) {
        let elems: *mut [T] = self.as_mut_slice();

        // SAFETY: copied from `Vec::clear`.
        unsafe {
            self.len = 0;
            ptr::drop_in_place(elems);
        }
    }

    #[inline(always)]
    fn insert(&mut self, offset: usize, child: T) {
        assert!(offset <= self.len());
        assert!(self.len() < N);

        let ptr = self.data.as_mut_ptr();

        // SAFETY: it's safe.
        unsafe {
            ptr::copy(
                ptr.add(offset),
                ptr.add(offset + 1),
                self.len() - offset,
            );
        };

        self.data[offset].write(child);

        self.len += 1;
    }

    #[inline(always)]
    fn into_vec(self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len() * 2);

        // SAFETY: it's safe.
        unsafe {
            let dest: *mut T = vec.as_mut_ptr();

            dest.copy_from_nonoverlapping(
                self.data.as_ptr() as *const T,
                self.len(),
            );

            vec.set_len(self.len());
        }

        core::mem::forget(self);

        vec
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        self.len() == N
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn remove(&mut self, idx: usize) {
        assert!(idx < self.len());

        // Read out the value at `idx` so it can be dropped.
        //
        // SAFETY: it's safe.
        let _ = unsafe { ptr::read(&self.data[idx]).assume_init() };

        let ptr = self.data.as_mut_ptr();

        // SAFETY: it's safe.
        unsafe {
            ptr::copy(ptr.add(idx + 1), ptr.add(idx), self.len() - idx - 1);
        }

        self.len -= 1;
    }
}

impl<const N: usize, T> Drop for InlineVec<N, T> {
    #[inline(always)]
    fn drop(&mut self) {
        for item in &mut self.data[..self.len] {
            // SAFETY: it's safe.
            unsafe { ptr::drop_in_place(item.as_mut_ptr()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_vec_spill_over() {
        let mut vec = TinyVec::<4, usize>::default();

        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len(), 4);
        assert!(vec.is_inline());

        vec.push(4);
        assert_eq!(vec.len(), 5);
        assert!(!vec.is_inline());
    }

    #[test]
    fn tiny_vec_spill_over_non_copy() {
        let mut vec = TinyVec::<4, String>::default();

        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
        vec.push(String::from("3"));
        assert_eq!(vec.len(), 4);
        assert!(vec.is_inline());

        vec.push(String::from("4"));
        assert_eq!(vec.len(), 5);
        assert!(!vec.is_inline());

        drop(vec);
    }

    #[test]
    fn tiny_vec_remove() {
        let mut vec = TinyVec::<4, String>::default();

        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
        vec.push(String::from("3"));
        vec.push(String::from("4"));

        assert_eq!(vec.len(), 5);
        assert!(!vec.is_inline());

        for _ in 0..5 {
            vec.remove(vec.len() - 1);
        }

        assert_eq!(vec.len(), 0);
    }
}
