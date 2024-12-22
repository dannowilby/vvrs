use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vvrs::chunk::{block::Block, mesher::mesh, Chunk, LocalBlockPos, CHUNK_SIZE};

fn create_random_chunk() -> Chunk {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if rand::random() {
                    let pos = LocalBlockPos(x, y, z);
                    chunk.set_block(pos, Block(1));
                }
            }
        }
    }

    chunk
}

fn create_full_chunk() -> Chunk {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let pos = LocalBlockPos(x, y, z);
                chunk.set_block(pos, Block(1));
            }
        }
    }

    chunk
}

fn build_and_mesh_chunk() {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if rand::random::<u8>() < 64 {
                    let pos = LocalBlockPos(x, y, z);
                    chunk.set_block(pos, Block(1));
                }
            }
        }
    }

    black_box(mesh(&chunk));
}

fn criterion_benchmark(c: &mut Criterion) {
    let chunk = create_random_chunk();
    c.bench_function("mesh random chunk", |b| b.iter(|| mesh(black_box(&chunk))));

    let chunk = create_full_chunk();
    c.bench_function("mesh full chunk", |b| b.iter(|| mesh(black_box(&chunk))));

    c.bench_function("build and mesh chunk", |b| b.iter(build_and_mesh_chunk));
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);
