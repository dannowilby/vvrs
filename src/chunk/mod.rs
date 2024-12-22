use std::collections::HashMap;

use block::Block;

pub mod block;
pub mod mesher;
pub mod pool;

/// This type indicates how large our chunks are. A ChunkDimTy of u32 indicates
/// that our chunk will be 32x32x32 blocks. We tie the type together with the
/// chunk size so that we can dynamically change the size without rewriting the
/// binary meshing algorithm. Any unsigned numeric natural type should work, but
/// other numeric types have not been tested yet.
pub type ChunkDimTy = u16;

/// Used for encoding the vertex position in a single vertex.
pub const NUM_BITS_IN_POS: ChunkDimTy =
    ChunkDimTy::ilog2(std::mem::size_of::<ChunkDimTy>() as ChunkDimTy) as ChunkDimTy;
pub const CHUNK_SIZE: ChunkDimTy = (std::mem::size_of::<ChunkDimTy>() * 8) as ChunkDimTy;

#[derive(Debug, Clone, Copy)]
pub struct EncodedVertex(pub u16);

impl EncodedVertex {
    pub fn to_untyped(&self) -> u16 {
        self.0
    }
}

/// The memory footprint of a maximal mesh, a proof will of which be provided in the docs.
pub const MAX_CHUNK_MEMORY_USAGE: u32 =
    (6 * std::mem::size_of::<EncodedVertex>() as u32) * (3 * CHUNK_SIZE.pow(3) as u32);
pub const MAX_UNIFORM_MEMORY_USAGE: u32 = (std::mem::size_of::<i32>() * 3) as u32;

/// Block position relative to the chunk.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct LocalBlockPos(pub ChunkDimTy, pub ChunkDimTy, pub ChunkDimTy);

impl LocalBlockPos {
    fn safe_sub(p1: &LocalBlockPos, p2: &LocalBlockPos) -> Option<LocalBlockPos> {
        let x = p1.0.checked_sub(p2.0);
        let y = p1.1.checked_sub(p2.1);
        let z = p1.2.checked_sub(p2.2);

        if x.is_none() || y.is_none() || z.is_none() {
            return None;
        }

        Some(LocalBlockPos(x.unwrap(), y.unwrap(), z.unwrap()))
    }
}

#[allow(dead_code)]
pub struct ChunkPos(i32, i32, i32);

#[derive(Default)]
pub struct Chunk {
    pub data: HashMap<LocalBlockPos, Block>,
}

/// Should add bounds checking for set/get
impl Chunk {
    pub fn get_block(&self, pos: &LocalBlockPos) -> Block {
        *self.data.get(pos).unwrap_or(&Block(0))
    }

    pub fn set_block(&mut self, pos: LocalBlockPos, b: Block) {
        self.data.insert(pos, b);
    }

    pub fn random() -> Self {
        let mut chunk = Chunk::default();

        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    if rand::random::<u8>() < 64 {
                        chunk.set_block(LocalBlockPos(i, j, k), Block(1));
                    }
                }
            }
        }

        chunk
    }
}
