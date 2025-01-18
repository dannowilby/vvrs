use std::collections::HashMap;

use wgpu::util::DrawIndirectArgs;

use crate::player::Player;

use super::pool::ChunkDrawInfo;

/// Traverses the world, queuing up the sides of chunks to be rendered
pub fn build_draw_list(
    lookup: &HashMap<(i32, i32, i32), ChunkDrawInfo>,
    _player: &Player,
) -> Vec<DrawIndirectArgs> {
    let mut indirect_data = vec![];

    // this is causing a significant slowdown
    for x in lookup.values() {
        let vertex_offset = x.vertex_offset;
        let storage_offset = x.storage_offset;

        // we are manually setting the all faces to be rendered
        for i in 0..6 {
            let face_offset = x.faces[i].0;
            let face_count = x.faces[i].1;
            indirect_data.push(DrawIndirectArgs {
                vertex_count: face_count,
                instance_count: 1,
                first_vertex: vertex_offset as u32 + face_offset,
                first_instance: storage_offset as u32, // use first instance to index into the uniform buffer
            });
        }
    }

    indirect_data
}
