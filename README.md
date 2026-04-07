# ris

[![Crates.io](https://img.shields.io/crates/v/ris.svg)](https://crates.io/crates/ris)
[![Docs.rs](https://docs.rs/ris/badge.svg)](https://docs.rs/ris)

A Rust crate to compute the Longest Increasing Subsequence (LIS) and other monotonic subsequences.

It runs in `O(N log N)` time and operates in `#![no_std]` environments (requires `alloc`).

## Features

- Computes the values, indices, or length of the longest increasing subsequence.
- Supports custom monotonic conditions, allowing you to extract strictly decreasing, non-decreasing, and non-increasing sequences.
- Provides extension traits for standard slices (`[T]`) and iterators.
- Requires no standard library (`#![no_std]`), utilizing only `alloc` for memory management.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ris = "0.1.0"
```

## Performance

The following benchmark compares `ris` with other LIS crates: [`longest-increasing-subsequence`](https://crates.io/crates/longest-increasing-subsequence) and [`lis`](https://crates.io/crates/lis).

![LIS Performance Benchmark](https://github.com/o24s/ris/blob/main/chart.svg)

## Usage

### Slice Extension

You can use the `LisExt` trait to compute subsequences directly on arrays and slices.

```rust
use ris::LisExt;

fn main() {
    let seq = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];

    // Get the values of the LIS
    let values = seq.lis_values();
    assert_eq!(values, [1, 2, 3, 5]);

    // Get the indices instead of the values
    let indices = seq.lis_indices();
    assert_eq!(indices, [3, 6, 9, 10]);

    // Get only the length
    let length = seq.lis_length();
    assert_eq!(length, 4);
}
```

### Other Monotonic Subsequences

The `LisExt` trait includes convenience methods to find other types of monotonic subsequences.

```rust
use ris::LisExt;

fn main() {
    let seq = [5, 5, 4, 3, 3, 2];

    // Longest Non-Increasing Subsequence (a >= b)
    let lnis = seq.lnis_values();
    assert_eq!(lnis, [5, 5, 4, 3, 3, 2]);

    // Longest Decreasing Subsequence (a > b)
    let lds = seq.lds_values();
    assert_eq!(lds, [5, 4, 3, 2]);
}
```

You can also provide a custom closure using the `_by` or `_by_key` methods.

```rust
use ris::LisExt;

fn main() {
    let seq = [10, 22, 9, 33, 21, 50, 41, 60];

    // Find the length based on a custom comparison closure
    let custom_len = seq.lis_length_by(|a, b| a < b);
    assert_eq!(custom_len, 5);
}
```

### Iterator Extension

The `IteratorLisExt` trait allows consuming an iterator directly to compute a subsequence.

```rust
use ris::IteratorLisExt;

fn main() {
    let seq = vec![3, 1, 4, 1, 5, 9];
    let values = seq.into_iter().lis();

    assert_eq!(values, [1, 4, 5, 9]);
}
```

## License

This project is dual-licensed under either the MIT license or the Apache License, Version 2.0, at your option.
