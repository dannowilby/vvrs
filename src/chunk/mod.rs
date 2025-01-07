use std::collections::HashMap;

use block::Block;

pub mod block;
pub mod manager;
pub mod mesher;
pub mod pool;

/// This type indicates how large our chunks are. A ChunkDimTy of u32 indicates
/// that our chunk will be 32x32x32 blocks. We tie the type together with the
/// chunk size so that we can dynamically change the size without rewriting the
/// binary meshing algorithm. Any unsigned numeric natural type should work, but
/// other numeric types have not been tested yet.
pub type ChunkDimTy = u32;

/// Used for encoding the vertex position in a single vertex. We add a bit to encompass when the vertex pos is the max (ie 32 in a 32 bit chunk won't fit inside 5 bits). This can probably be changed if padding is added.
pub const NUM_BITS_IN_POS: ChunkDimTy =
    ChunkDimTy::ilog2(8 * std::mem::size_of::<ChunkDimTy>() as ChunkDimTy) + 1 as ChunkDimTy;
pub const CHUNK_SIZE: ChunkDimTy = (std::mem::size_of::<ChunkDimTy>() * 8) as ChunkDimTy;

#[derive(Debug, Clone, Copy)]
pub struct EncodedVertex(pub u32);

// Would probably be better to do things with this at some point
// impl Deref for EncodedVertex {
//     type Target = u32;

//     fn deref(&self) -> &Self::Target {
//         todo!()
//     }
// }

impl EncodedVertex {
    pub fn to_untyped(&self) -> u32 {
        self.0
    }
}

/// Block position relative to the chunk.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
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
                    if rand::random::<u8>() < u8::MAX - 1 {
                        chunk.set_block(LocalBlockPos(i, j, k), Block(1));
                    }
                }
            }
        }

        chunk
    }
}
