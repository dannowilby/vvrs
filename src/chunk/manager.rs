use std::collections::HashSet;

use crate::{chunk::ChunkPos, player::Player, window_state::WindowState};

use super::{pool::ChunkPool, Chunk};

#[derive(Default)]
pub struct ChunkManager {
    pool: ChunkPool,
    loaded_chunks: HashSet<ChunkPos>,
}

impl ChunkManager {
    pub fn init(&mut self, state: &WindowState) {
        self.pool = ChunkPool::initialize(state);
    }

    /// Recalculates the chunks that need to be loaded, and loads them.
    pub fn load_chunks(&mut self, state: &WindowState, player: &Player) {
        let mut chunks_to_remove: HashSet<_> = self.loaded_chunks.iter().cloned().collect();
        let mut chunks_to_add = Vec::<ChunkPos>::new();

        let pos = player.get_chunk_pos();
        let r = player.load_radius as i32;

        for x in (pos.0 - r)..=(pos.0 + r) {
            for y in (pos.0 - r)..=(pos.0 + r) {
                for z in (pos.0 - r)..=(pos.0 + r) {
                    let new_pos = ChunkPos(x, y, z);

                    if !self.loaded_chunks.contains(&new_pos) {
                        chunks_to_add.push(new_pos);
                    }

                    chunks_to_remove.remove(&new_pos);
                }
            }
        }

        // remove the chunks and add their memory address to the free list
        for chunk_pos in chunks_to_remove {
            self.pool.remove_chunk(chunk_pos);
        }

        for chunk_pos in chunks_to_add {
            self.pool.add_chunk(state, chunk_pos, Chunk::full());
            self.loaded_chunks.insert(chunk_pos);
        }

        let [x, y] = self.pool.allocated_percent();
        log::info!("Chunk manager statistics ----");
        log::info!("Number of loaded chunks: {}", self.loaded_chunks.len());
        log::info!("Vertex buffer usage: {:.2}%", 100.0 * x);
        log::info!("Storage buffer usage: {:.2}%", 100.0 * y)
    }

    pub fn render(&self, state: &WindowState, player: &Player) {
        self.pool.render(state, player, ());
    }
}
