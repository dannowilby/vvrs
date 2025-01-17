use std::time::Duration;

use criterion::{black_box, criterion_group, Criterion};
use vvrs::chunk::{culling::VisibilityGraph, Chunk};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("visibility graph overhead");

    let chunk = Chunk::random();
    group.bench_function("build random chunk visibility graph", |b| {
        b.iter(|| black_box(VisibilityGraph::from_chunk(&chunk)))
    });

    let chunk = Chunk::full();
    group.bench_function("build full chunk visibility graph", |b| {
        b.iter(|| black_box(VisibilityGraph::from_chunk(&chunk)))
    });

    let chunk = Chunk::default();
    group.bench_function("build empty chunk visibility graph", |b| {
        b.iter(|| black_box(VisibilityGraph::from_chunk(&chunk)))
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
