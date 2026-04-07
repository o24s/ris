//! Computes the Longest Increasing Subsequence (LIS) and other monotonic subsequences.
//!
//! This crate provides an algorithm with `O(N log N)` time complexity to find the longest
//! increasing, decreasing, or custom monotonic subsequence.
//!
//! # Examples
//!
//! Using the slice extension trait `LisExt`:
//!
//! ```rust
//! use ris::LisExt;
//!
//! let seq = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
//!
//! // Get the indices of the LIS
//! let indices = seq.lis_indices();
//! assert_eq!(indices, [3, 6, 9, 10]);
//!
//! // Get the values directly
//! let values = seq.lis_values();
//! assert_eq!(values, [1, 2, 3, 5]);
//!
//! // Get the length only
//! let length = seq.lis_length();
//! assert_eq!(length, 4);
//! ```
//!
//! Using the iterator extension trait `IteratorLisExt`:
//!
//! ```rust
//! use ris::IteratorLisExt;
//!
//! let seq = [3, 1, 4, 1, 5, 9];
//! let values = seq.into_iter().lis();
//! assert_eq!(values, [1, 4, 5, 9]);
//! ```

#![no_std]

extern crate alloc;

#[cfg(feature = "diff")]
pub mod diff;
#[cfg(feature = "diff")]
pub use diff::{DiffCallback, diff_by_key};

use alloc::vec::Vec;
use core::hint::assert_unchecked;

/// Computes the indices of the longest subsequence that satisfies the `is_less` condition.
///
/// This function allocates memory to track the predecessors of each element to reconstruct
/// the sequence.
///
/// # Arguments
///
/// - `items`: A slice of elements to process.
/// - `is_less`: A closure that defines the strictly increasing condition between two elements.
///
/// # Examples
///
/// ```rust
/// use ris::lis;
///
/// let arr = [10, 22, 9, 33, 21, 50, 41, 60];
/// let indices = lis(&arr, |a, b| a < b);
/// assert_eq!(indices, [0, 1, 3, 6, 7]);
/// ```
pub fn lis<T, F>(items: &[T], mut is_less: F) -> Vec<usize>
where
    F: FnMut(&T, &T) -> bool,
{
    let len = items.len();

    if len == 0 {
        return Vec::new();
    }

    let mut tails: Vec<usize> = Vec::with_capacity(len);
    let mut preds: Vec<usize> = Vec::with_capacity(len);

    let tails_ptr = tails.as_mut_ptr();
    let preds_ptr = preds.as_mut_ptr();
    let items_ptr = items.as_ptr();

    unsafe {
        tails_ptr.write(0);
        preds_ptr.write(usize::MAX);

        let k = if len <= 2048 {
            lis_inner_in_cache_search(len, items_ptr, tails_ptr, preds_ptr, &mut is_less)
        } else {
            lis_inner_partition_point(len, items_ptr, tails_ptr, preds_ptr, &mut is_less)
        };

        let lis_len = k + 1;
        let mut result: Vec<usize> = Vec::with_capacity(lis_len);

        let mut curr = *tails_ptr.add(k);
        let mut res_ptr = result.as_mut_ptr().add(lis_len);

        for _ in 0..lis_len {
            res_ptr = res_ptr.sub(1);
            res_ptr.write(curr);
            curr = *preds_ptr.add(curr);
        }

        result.set_len(lis_len);
        tails.set_len(lis_len);
        preds.set_len(len);

        result
    }
}

/// Computes only the length of the longest subsequence that satisfies the `is_less` condition.
///
/// This avoids allocating memory for the predecessor array and skips the sequence
/// reconstruction phase.
///
/// # Arguments
///
/// - `items`: A slice of elements to process.
/// - `is_less`: A closure that defines the strictly increasing condition between two elements.
///
/// # Examples
///
/// ```rust
/// use ris::lis_length;
///
/// let arr = [10, 22, 9, 33, 21, 50, 41, 60];
/// let length = lis_length(&arr, |a, b| a < b);
/// assert_eq!(length, 5);
/// ```
pub fn lis_length<T, F>(items: &[T], mut is_less: F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let len = items.len();
    if len <= 1 {
        return len;
    }

    let mut tails: Vec<usize> = Vec::with_capacity(len);
    let tails_ptr = tails.as_mut_ptr();
    let items_ptr = items.as_ptr();

    unsafe {
        tails_ptr.write(0);

        let k = if len <= 2048 {
            lis_length_inner_in_cache_search(len, items_ptr, tails_ptr, &mut is_less)
        } else {
            lis_length_inner_partition_point(len, items_ptr, tails_ptr, &mut is_less)
        };

        k + 1
    }
}

#[inline(always)]
unsafe fn lis_inner_in_cache_search<T, F>(
    len: usize,
    items_ptr: *const T,
    tails_ptr: *mut usize,
    preds_ptr: *mut usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    unsafe {
        let mut k = 0;
        for i in 1..len {
            let last_tail = *tails_ptr.add(k);
            assert_unchecked(last_tail < len);
            let x = &*items_ptr.add(i);

            if is_less(&*items_ptr.add(last_tail), x) {
                k += 1;
                tails_ptr.add(k).write(i);
                preds_ptr.add(i).write(last_tail);
                continue;
            }

            let mut lo = 0;
            let mut hi = k;
            while lo < hi {
                let mid = (lo + hi) >> 1;
                let t = *tails_ptr.add(mid);
                assert_unchecked(t < len);

                let less = is_less(&*items_ptr.add(t), x);
                lo = if less { mid + 1 } else { lo };
                hi = if less { hi } else { mid };
            }

            let p_idx = if lo > 0 { lo - 1 } else { 0 };
            let p_val = *tails_ptr.add(p_idx);
            let pred = if lo > 0 { p_val } else { usize::MAX };

            preds_ptr.add(i).write(pred);
            tails_ptr.add(lo).write(i);
        }
        k
    }
}

#[inline(always)]
unsafe fn lis_inner_partition_point<T, F>(
    len: usize,
    items_ptr: *const T,
    tails_ptr: *mut usize,
    preds_ptr: *mut usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    unsafe {
        let mut k = 0;
        for i in 1..len {
            let last_tail = *tails_ptr.add(k);
            assert_unchecked(last_tail < len);
            let x = &*items_ptr.add(i);

            if is_less(&*items_ptr.add(last_tail), x) {
                k += 1;
                tails_ptr.add(k).write(i);
                preds_ptr.add(i).write(last_tail);
                continue;
            }

            let search_slice = core::slice::from_raw_parts(tails_ptr, k);
            let pos = search_slice.partition_point(|&t| is_less(&*items_ptr.add(t), x));

            let pred = if pos > 0 {
                *tails_ptr.add(pos - 1)
            } else {
                usize::MAX
            };

            preds_ptr.add(i).write(pred);
            tails_ptr.add(pos).write(i);
        }
        k
    }
}

#[inline(always)]
unsafe fn lis_length_inner_in_cache_search<T, F>(
    len: usize,
    items_ptr: *const T,
    tails_ptr: *mut usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    unsafe {
        let mut k = 0;
        for i in 1..len {
            let last_tail = *tails_ptr.add(k);
            assert_unchecked(last_tail < len);
            let x = &*items_ptr.add(i);

            if is_less(&*items_ptr.add(last_tail), x) {
                k += 1;
                tails_ptr.add(k).write(i);
                continue;
            }

            let mut lo = 0;
            let mut hi = k;
            while lo < hi {
                let mid = (lo + hi) >> 1;
                let t = *tails_ptr.add(mid);
                assert_unchecked(t < len);

                let less = is_less(&*items_ptr.add(t), x);
                lo = if less { mid + 1 } else { lo };
                hi = if less { hi } else { mid };
            }

            tails_ptr.add(lo).write(i);
        }
        k
    }
}

#[inline(always)]
unsafe fn lis_length_inner_partition_point<T, F>(
    len: usize,
    items_ptr: *const T,
    tails_ptr: *mut usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    unsafe {
        let mut k = 0;
        for i in 1..len {
            let last_tail = *tails_ptr.add(k);
            assert_unchecked(last_tail < len);
            let x = &*items_ptr.add(i);

            if is_less(&*items_ptr.add(last_tail), x) {
                k += 1;
                tails_ptr.add(k).write(i);
                continue;
            }

            let search_slice = core::slice::from_raw_parts(tails_ptr, k);
            let pos = search_slice.partition_point(|&t| is_less(&*items_ptr.add(t), x));

            tails_ptr.add(pos).write(i);
        }
        k
    }
}

/// Extension trait to compute longest monotonic subsequences on slices.
pub trait LisExt<T> {
    /// Computes the indices of the longest strictly increasing subsequence.
    fn lis_indices(&self) -> Vec<usize>
    where
        T: Ord;

    /// Computes the indices of the longest subsequence based on a custom `is_less` closure.
    fn lis_indices_by<F>(&self, is_less: F) -> Vec<usize>
    where
        F: FnMut(&T, &T) -> bool;

    /// Computes the indices of the longest subsequence based on a key extraction closure.
    fn lis_indices_by_key<K, F>(&self, f: F) -> Vec<usize>
    where
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the values of the longest strictly increasing subsequence.
    fn lis_values(&self) -> Vec<T>
    where
        T: Ord + Clone;

    /// Computes the values of the longest subsequence based on a custom `is_less` closure.
    fn lis_values_by<F>(&self, is_less: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T, &T) -> bool;

    /// Computes the values of the longest subsequence based on a key extraction closure.
    fn lis_values_by_key<K, F>(&self, f: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the references of the longest strictly increasing subsequence.
    fn lis_refs(&self) -> Vec<&T>
    where
        T: Ord;

    /// Computes the references of the longest subsequence based on a custom `is_less` closure.
    fn lis_refs_by<F>(&self, is_less: F) -> Vec<&T>
    where
        F: FnMut(&T, &T) -> bool;

    /// Computes the references of the longest subsequence based on a key extraction closure.
    fn lis_refs_by_key<K, F>(&self, f: F) -> Vec<&T>
    where
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the length of the longest strictly increasing subsequence.
    fn lis_length(&self) -> usize
    where
        T: Ord;

    /// Computes the length of the longest subsequence based on a custom `is_less` closure.
    fn lis_length_by<F>(&self, is_less: F) -> usize
    where
        F: FnMut(&T, &T) -> bool;

    /// Computes the length of the longest subsequence based on a key extraction closure.
    fn lis_length_by_key<K, F>(&self, f: F) -> usize
    where
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the indices of the longest subsequence based on a cached key extraction closure.
    /// This is faster than `lis_indices_by_key` when the key extraction is expensive.
    fn lis_indices_by_cached_key<K, F>(&self, f: F) -> Vec<usize>
    where
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the values of the longest subsequence based on a cached key extraction closure.
    fn lis_values_by_cached_key<K, F>(&self, f: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the references of the longest subsequence based on a cached key extraction closure.
    fn lis_refs_by_cached_key<K, F>(&self, f: F) -> Vec<&T>
    where
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the length of the longest subsequence based on a cached key extraction closure.
    fn lis_length_by_cached_key<K, F>(&self, f: F) -> usize
    where
        F: FnMut(&T) -> K,
        K: Ord;

    /// Computes the values of the longest strictly decreasing subsequence.
    fn lds_values(&self) -> Vec<T>
    where
        T: Ord + Clone;

    /// Computes the values of the longest non-decreasing subsequence.
    fn lnds_values(&self) -> Vec<T>
    where
        T: Ord + Clone;

    /// Computes the values of the longest non-increasing subsequence.
    fn lnis_values(&self) -> Vec<T>
    where
        T: Ord + Clone;
}

impl<T> LisExt<T> for [T] {
    #[inline]
    fn lis_indices(&self) -> Vec<usize>
    where
        T: Ord,
    {
        lis(self, |a, b| a < b)
    }

    #[inline]
    fn lis_indices_by<F>(&self, is_less: F) -> Vec<usize>
    where
        F: FnMut(&T, &T) -> bool,
    {
        lis(self, is_less)
    }

    #[inline]
    fn lis_indices_by_key<K, F>(&self, mut f: F) -> Vec<usize>
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        lis(self, |a, b| f(a) < f(b))
    }

    #[inline]
    fn lis_values(&self) -> Vec<T>
    where
        T: Ord + Clone,
    {
        self.lis_indices()
            .into_iter()
            .map(|i| self[i].clone())
            .collect()
    }

    #[inline]
    fn lis_values_by<F>(&self, is_less: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T, &T) -> bool,
    {
        self.lis_indices_by(is_less)
            .into_iter()
            .map(|i| self[i].clone())
            .collect()
    }

    #[inline]
    fn lis_values_by_key<K, F>(&self, f: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.lis_indices_by_key(f)
            .into_iter()
            .map(|i| self[i].clone())
            .collect()
    }

    #[inline]
    fn lis_refs(&self) -> Vec<&T>
    where
        T: Ord,
    {
        self.lis_indices().into_iter().map(|i| &self[i]).collect()
    }

    #[inline]
    fn lis_refs_by<F>(&self, is_less: F) -> Vec<&T>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.lis_indices_by(is_less)
            .into_iter()
            .map(|i| &self[i])
            .collect()
    }

    #[inline]
    fn lis_refs_by_key<K, F>(&self, f: F) -> Vec<&T>
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.lis_indices_by_key(f)
            .into_iter()
            .map(|i| &self[i])
            .collect()
    }

    #[inline]
    fn lis_length(&self) -> usize
    where
        T: Ord,
    {
        lis_length(self, |a, b| a < b)
    }

    #[inline]
    fn lis_length_by<F>(&self, is_less: F) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        lis_length(self, is_less)
    }

    #[inline]
    fn lis_length_by_key<K, F>(&self, mut f: F) -> usize
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        lis_length(self, |a, b| f(a) < f(b))
    }

    #[inline]
    fn lds_values(&self) -> Vec<T>
    where
        T: Ord + Clone,
    {
        self.lis_values_by(|a, b| a > b)
    }

    #[inline]
    fn lnds_values(&self) -> Vec<T>
    where
        T: Ord + Clone,
    {
        self.lis_values_by(|a, b| a <= b)
    }

    #[inline]
    fn lnis_values(&self) -> Vec<T>
    where
        T: Ord + Clone,
    {
        self.lis_values_by(|a, b| a >= b)
    }

    #[inline]
    fn lis_indices_by_cached_key<K, F>(&self, mut f: F) -> Vec<usize>
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        let keys: Vec<K> = self.iter().map(&mut f).collect();
        lis(&keys, |a, b| a < b)
    }

    #[inline]
    fn lis_values_by_cached_key<K, F>(&self, f: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.lis_indices_by_cached_key(f)
            .into_iter()
            .map(|i| self[i].clone())
            .collect()
    }

    #[inline]
    fn lis_refs_by_cached_key<K, F>(&self, f: F) -> Vec<&T>
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.lis_indices_by_cached_key(f)
            .into_iter()
            .map(|i| &self[i])
            .collect()
    }

    #[inline]
    fn lis_length_by_cached_key<K, F>(&self, mut f: F) -> usize
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        let keys: Vec<K> = self.iter().map(&mut f).collect();
        lis_length(&keys, |a, b| a < b)
    }
}

/// Extension trait to compute longest monotonic subsequences on iterators.
pub trait IteratorLisExt: Iterator {
    /// Consumes the iterator and computes the values of the longest strictly increasing subsequence.
    fn lis(self) -> Vec<Self::Item>
    where
        Self: Sized,
        Self::Item: Ord,
    {
        let mut items: Vec<Self::Item> = self.collect();
        let indices = items.lis_indices();

        for (dest, &src) in indices.iter().enumerate() {
            items.swap(dest, src);
        }
        items.truncate(indices.len());
        items
    }

    /// Consumes the iterator and computes the values based on a custom `is_less` closure.
    fn lis_by<F>(self, is_less: F) -> Vec<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> bool,
    {
        let mut items: Vec<Self::Item> = self.collect();
        let indices = items.lis_indices_by(is_less);

        for (dest, &src) in indices.iter().enumerate() {
            items.swap(dest, src);
        }
        items.truncate(indices.len());
        items
    }

    /// Consumes the iterator and computes the values based on a key extraction closure.
    fn lis_by_key<K, F>(self, f: F) -> Vec<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: Ord,
    {
        let mut items: Vec<Self::Item> = self.collect();
        let indices = items.lis_indices_by_key(f);

        for (dest, &src) in indices.iter().enumerate() {
            items.swap(dest, src);
        }
        items.truncate(indices.len());
        items
    }

    /// Consumes the iterator and computes the values based on a cached key extraction closure.
    fn lis_by_cached_key<K, F>(self, f: F) -> Vec<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: Ord,
    {
        let mut items: Vec<Self::Item> = self.collect();
        let indices = items.lis_indices_by_cached_key(f);

        for (dest, &src) in indices.iter().enumerate() {
            items.swap(dest, src);
        }
        items.truncate(indices.len());
        items
    }

    /// Consumes the iterator and computes the length of the longest strictly increasing subsequence.
    fn lis_length(self) -> usize
    where
        Self: Sized,
        Self::Item: Ord,
    {
        let items: Vec<Self::Item> = self.collect();
        items.lis_length()
    }

    /// Consumes the iterator and computes the length based on a custom `is_less` closure.
    fn lis_length_by<F>(self, is_less: F) -> usize
    where
        Self: Sized,
        F: FnMut(&Self::Item, &Self::Item) -> bool,
    {
        let items: Vec<Self::Item> = self.collect();
        items.lis_length_by(is_less)
    }

    /// Consumes the iterator and computes the length based on a key extraction closure.
    fn lis_length_by_key<K, F>(self, f: F) -> usize
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: Ord,
    {
        let items: Vec<Self::Item> = self.collect();
        items.lis_length_by_key(f)
    }

    /// Consumes the iterator and computes the length based on a cached key extraction closure.
    fn lis_length_by_cached_key<K, F>(self, f: F) -> usize
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: Ord,
    {
        let items: Vec<Self::Item> = self.collect();
        items.lis_length_by_cached_key(f)
    }
}

impl<I: Iterator> IteratorLisExt for I {}
