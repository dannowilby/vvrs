use std::collections::HashMap;

use block::Block;

pub mod block;
pub mod mesher;

pub const CHUNK_SIZE: usize = 32;

// May want to consider storing padding by default, which also means updating
// the meshing code for this (should be easy, would just need to disregard the
// most/least significant bits of the mask)
#[derive(Default)]
pub struct Chunk {
    pub data: HashMap<(u32, u32, u32), Block>,
}

/// Should add bounds checking for set/get
impl Chunk {
    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Block {
        *self.data.get(&(x, y, z)).unwrap_or(&Block(0))
    }

    pub fn set_block(&mut self, x: u32, y: u32, z: u32, b: Block) {
        self.data.insert((x, y, z), b);
    }
}
