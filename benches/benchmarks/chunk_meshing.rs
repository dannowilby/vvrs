use std::time::Duration;

use criterion::{black_box, criterion_group, Criterion};
use vvrs::chunk::{mesher::mesh, Chunk};

fn build_and_mesh_chunk() {
    let chunk = Chunk::random();

    black_box(mesh(&chunk));
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk meshing overhead");

    let chunk = Chunk::random();
    group.bench_function("mesh random chunk", |b| b.iter(|| black_box(mesh(&chunk))));

    let chunk = Chunk::full();
    group.bench_function("mesh full chunk", |b| b.iter(|| black_box(mesh(&chunk))));

    group.bench_function("build and mesh chunk", |b| b.iter(build_and_mesh_chunk));

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(15));
    targets = criterion_benchmark
}
