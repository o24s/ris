use crate::LisExt;
use alloc::vec;
use alloc::vec::Vec;
use core::hash::Hash;
use rustc_hash::FxHashMap;

pub trait DiffCallback<O: ?Sized, N: ?Sized> {
    fn inserted(&mut self, new: &N);
    fn unchanged(&mut self, old: &O, new: &N);
    fn removed(&mut self, old: &O);
    fn moved(&mut self, old: &O, new: &N);
}

pub fn diff_by_key<O, N, K, F, G, C>(
    old_items: &[O],
    mut old_key_fn: F,
    new_items: &[N],
    mut new_key_fn: G,
    cb: &mut C,
) where
    K: Eq + Hash,
    F: FnMut(&O) -> K,
    G: FnMut(&N) -> K,
    C: DiffCallback<O, N>,
{
    let old_len = old_items.len();
    let new_len = new_items.len();

    let mut prefix_len = 0;
    while prefix_len < old_len && prefix_len < new_len {
        let o = &old_items[prefix_len];
        let n = &new_items[prefix_len];
        if old_key_fn(o) == new_key_fn(n) {
            cb.unchanged(o, n);
            prefix_len += 1;
        } else {
            break;
        }
    }

    let mut suffix_len = 0;
    while prefix_len + suffix_len < old_len && prefix_len + suffix_len < new_len {
        let o = &old_items[old_len - 1 - suffix_len];
        let n = &new_items[new_len - 1 - suffix_len];
        if old_key_fn(o) == new_key_fn(n) {
            cb.unchanged(o, n);
            suffix_len += 1;
        } else {
            break;
        }
    }

    let old_middle = &old_items[prefix_len..old_len - suffix_len];
    let new_middle = &new_items[prefix_len..new_len - suffix_len];
    let new_middle_len = new_middle.len();

    let mut old_indices = vec![usize::MAX; new_middle_len];
    let mut is_moved = false;
    let mut last_new_idx = 0;

    if new_middle_len < 32 {
        for (old_idx, o) in old_middle.iter().enumerate() {
            let key = old_key_fn(o);
            let mut found = false;
            for (new_idx, n) in new_middle.iter().enumerate() {
                if new_key_fn(n) == key {
                    old_indices[new_idx] = old_idx;
                    if new_idx < last_new_idx {
                        is_moved = true;
                    } else {
                        last_new_idx = new_idx;
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                cb.removed(o);
            }
        }
    } else {
        let mut new_key_to_idx =
            FxHashMap::with_capacity_and_hasher(new_middle_len, Default::default());
        for (i, n) in new_middle.iter().enumerate() {
            new_key_to_idx.insert(new_key_fn(n), i);
        }

        for (old_idx, o) in old_middle.iter().enumerate() {
            let key = old_key_fn(o);
            if let Some(&new_idx) = new_key_to_idx.get(&key) {
                old_indices[new_idx] = old_idx;
                if new_idx < last_new_idx {
                    is_moved = true;
                } else {
                    last_new_idx = new_idx;
                }
            } else {
                cb.removed(o);
            }
        }
    }

    let mut lis_new_indices = Vec::new();

    if is_moved {
        let mut present = Vec::with_capacity(new_middle_len);
        for (new_idx, &old_idx) in old_indices.iter().enumerate() {
            if old_idx != usize::MAX {
                present.push((new_idx, old_idx));
            }
        }

        let lis_indices = present.lis_indices_by(|a, b| a.1 < b.1);
        for &i in &lis_indices {
            lis_new_indices.push(present[i].0);
        }
    } else {
        for (new_idx, &old_idx) in old_indices.iter().enumerate() {
            if old_idx != usize::MAX {
                lis_new_indices.push(new_idx);
            }
        }
    }

    for new_idx in (0..new_middle_len).rev() {
        let n = &new_middle[new_idx];
        let old_idx = old_indices[new_idx];

        if old_idx != usize::MAX {
            let o = &old_middle[old_idx];
            if lis_new_indices.last() == Some(&new_idx) {
                lis_new_indices.pop();
                cb.unchanged(o, n);
            } else {
                cb.moved(o, n);
            }
        } else {
            cb.inserted(n);
        }
    }
}
