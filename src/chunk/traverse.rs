use std::collections::HashMap;

use cgmath::Matrix;
use wgpu::util::DrawIndirectArgs;

use crate::player::Player;

use super::{pool::ChunkDrawInfo, ChunkPos, CHUNK_SIZE};

/// Traverses the world, queuing up the sides of chunks to be rendered
pub fn build_draw_list(
    lookup: &HashMap<ChunkPos, ChunkDrawInfo>,
    player: &Player,
) -> Vec<DrawIndirectArgs> {
    let mut indirect_data = vec![];

    let frustum_planes = calculate_frustum_planes(player);

    for (pos, x) in lookup.iter() {
        let vertex_offset = x.vertex_offset;
        let storage_offset = x.storage_offset;

        if is_chunk_inside_frustum(*pos, frustum_planes) {
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
    }

    indirect_data
}

fn calculate_frustum_planes(player: &Player) -> [cgmath::Vector4<f32>; 6] {
    let pvm = player.get_projection() * player.get_view();

    let row0 = pvm.row(0);
    let row1 = pvm.row(1);
    let row2 = pvm.row(2);
    let row3 = pvm.row(3);

    [
        row3 + row0,
        row3 - row0,
        row3 + row1,
        row3 - row1,
        row3 + row2,
        row3 - row2,
    ]
}

/// Frustum cull if chunk is completely outside of frustum.
/// Code is a mix of ChatGPT code and the article found [here](https://iquilezles.org/articles/frustumcorrect/).
fn is_chunk_inside_frustum(chunk_pos: ChunkPos, frustum_planes: [cgmath::Vector4<f32>; 6]) -> bool {
    let min = (
        chunk_pos.0 * CHUNK_SIZE as i32,
        chunk_pos.1 * CHUNK_SIZE as i32,
        chunk_pos.2 * CHUNK_SIZE as i32,
    );
    let max = (
        chunk_pos.0 * CHUNK_SIZE as i32 + CHUNK_SIZE as i32,
        chunk_pos.1 * CHUNK_SIZE as i32 + CHUNK_SIZE as i32,
        chunk_pos.2 * CHUNK_SIZE as i32 + CHUNK_SIZE as i32,
    );

    for plane in frustum_planes {
        let mut output = 0;

        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(min.0 as f32, min.1 as f32, min.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(max.0 as f32, min.1 as f32, min.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(min.0 as f32, max.1 as f32, min.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(min.0 as f32, min.1 as f32, max.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(max.0 as f32, max.1 as f32, min.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(max.0 as f32, min.1 as f32, max.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(min.0 as f32, max.1 as f32, max.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }
        if cgmath::dot(
            plane,
            cgmath::Vector4::<f32>::new(max.0 as f32, max.1 as f32, max.2 as f32, 1.0),
        ) < 0.0
        {
            output += 1;
        }

        if output == 8 {
            return false;
        }
    }
    true
}
