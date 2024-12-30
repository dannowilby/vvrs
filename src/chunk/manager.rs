
struct ChunkManager {
    pool: ChunkPool,
    loaded_chunks: HashSet<(i32, i32, i32)>
}

/// Recalculates the chunks that need to be loaded, and loads them.
pub fn load_chunks(&mut self, state: &WindowState, player: &Player) {
    let mut chunks_to_remove: HashSet<(i32, i32, i32)> = self.lookup.keys().cloned().collect();
    let mut chunks_to_add = Vec::<(i32, i32, i32)>::new();

    let pos = player.get_chunk_pos();
    let r = player.load_radius as i32;

    for x in (pos.0 - r)..=(pos.0 + r) {
        for y in (pos.0 - r)..=(pos.0 + r) {
            for z in (pos.0 - r)..=(pos.0 + r) {
                let new_pos = (x, y, z);

                if !self.lookup.contains_key(&new_pos) {
                    chunks_to_add.push(new_pos);
                }

                chunks_to_remove.remove(&new_pos);
            }
        }
    }

    // remove the chunks and add their memory address to the free list
    for chunk_pos in chunks_to_remove {
        let Some(chunk_info) = self.lookup.get(&chunk_pos) else {
            continue;
        };

        self.allocator.dealloc(chunk_info.offset);
        self.lookup.remove(&chunk_pos);
    }

    for chunk_pos in chunks_to_add {
        self.upload_chunk(state, chunk_pos);
    }
}