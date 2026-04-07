use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::RngExt;

fn generate_random_vec(size: usize) -> Vec<i32> {
    let mut rng = rand::rng();
    (0..size).map(|_| rng.random_range(0..10_000)).collect()
}

fn bench_lis(c: &mut Criterion) {
    let mut group = c.benchmark_group("LIS Comparison");

    let sizes = [16, 64, 256, 1024, 10_000, 100_000];

    for &size in sizes.iter() {
        let data = generate_random_vec(size);

        group.bench_with_input(BenchmarkId::new("ris", size), &data, |b, items| {
            use ris::LisExt;
            b.iter(|| black_box(items.lis_indices()))
        });

        group.bench_with_input(
            BenchmarkId::new("longest-increasing-subsequence", size),
            &data,
            |b, items| b.iter(|| black_box(longest_increasing_subsequence::lis(items))),
        );

        group.bench_with_input(BenchmarkId::new("lis", size), &data, |b, items| {
            use lis::LisExt;
            b.iter(|| black_box(items.longest_increasing_subsequence()))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_lis);
criterion_main!(benches);
