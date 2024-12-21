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

fn criterion_benchmark(c: &mut Criterion) {
    let chunk = create_random_chunk();
    c.bench_function("mesh random chunk", |b| b.iter(|| mesh(black_box(&chunk))));

    let chunk = create_full_chunk();
    c.bench_function("mesh full chunk", |b| b.iter(|| mesh(black_box(&chunk))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
