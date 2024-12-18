use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vvrs::chunk::{block::Block, mesher::mesh, Chunk, CHUNK_SIZE};

fn create_random_chunk() -> Chunk {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if rand::random() {
                    chunk.set_block(x as u32, y as u32, z as u32, Block(1));
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
                chunk.set_block(x as u32, y as u32, z as u32, Block(1));
            }
        }
    }

    chunk
}

fn criterion_benchmark(c: &mut Criterion) {
    let chunk = create_random_chunk();
    c.bench_function("mesh random chunk", |b| b.iter(|| mesh(black_box(&chunk))));

    let chunk = create_full_chunk();
    c.bench_function("mesh full chunk", |b| b.iter(|| mesh(black_box(&chunk))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
