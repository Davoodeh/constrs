use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::{collections::HashSet, hint::black_box};

use constrs::date::u32::{
    is_leap, next_leap, previous_leap, timestamp, timestamp_i64, timestamp_u64,
};

fn next_leap_max(c: &mut Criterion) {
    c.bench_function("next_leap_max", |b| {
        b.iter(|| next_leap(black_box(u32::MAX - 7)))
    });
}

fn previous_leap_max(c: &mut Criterion) {
    c.bench_function("previous_leap_max", |b| {
        b.iter(|| previous_leap(black_box(u32::MAX - 7)))
    });
}

fn is_leap_max(c: &mut Criterion) {
    c.bench_function("is_leap_max", |b| {
        b.iter(|| is_leap(black_box(u32::MAX - 7)))
    });
}

// currently it takes roughlty 216 second (100 iter) on an i7-10700T (16) @ 4.50 Ghz
// meaning each percentage of u32::MAX takes almost 20ms for each call making the final call 2.16s
//
// flamegraph data shows next_leap consumes 75% of the time.
fn timestamp_max(c: &mut Criterion) {
    c.bench_function("timestamp_max", |b| {
        b.iter(|| timestamp_i64(black_box(u32::MAX), black_box(12), black_box(31)))
    });
}

fn timestamp_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_range");
    let mut ids = HashSet::with_capacity(100);
    for mul in 0..10 {
        let year = (((u32::MAX as u64) * mul) / 100) as u32;
        if !ids.insert(year) {
            continue;
        }
        group.bench_with_input(BenchmarkId::from_parameter(year), &year, |b, &year| {
            b.iter(|| timestamp_i64(black_box(year), black_box(12), black_box(31)));
        });
    }
    group.finish();
}

fn timestamp_u32_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_u32_u64");
    for i in [1970, 2010, 2050, 2080, 2100].iter() {
        group.bench_with_input(BenchmarkId::new("timestamp_u64", i), i, |b, i| {
            b.iter(|| timestamp_u64(*i, 12, 31))
        });
        group.bench_with_input(BenchmarkId::new("timestamp_u32", i), i, |b, i| {
            b.iter(|| timestamp(*i, 12, 31))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    timestamp_max,
    timestamp_range,
    next_leap_max,
    previous_leap_max,
    is_leap_max,
    timestamp_u32_u64,
);
criterion_main!(benches);
