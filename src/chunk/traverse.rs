use std::collections::{HashMap, HashSet, VecDeque};

use cgmath::Matrix;
use wgpu::util::DrawIndirectArgs;

use crate::player::Player;

use super::{pool::ChunkDrawInfo, visibility::Side, ChunkPos, CHUNK_SIZE};

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

fn create_chunk_indirect_args(draw_info: &ChunkDrawInfo, side: Side) -> DrawIndirectArgs {
    let face_count = draw_info.faces[side as usize].1;
    let vertex_offset = draw_info.vertex_offset;
    let face_offset = draw_info.faces[side as usize].0;
    let storage_offset = draw_info.storage_offset;

    DrawIndirectArgs {
        vertex_count: face_count,
        instance_count: 1,
        first_vertex: vertex_offset as u32 + face_offset,
        first_instance: storage_offset as u32, // use first instance to index into the uniform buffer
    }
}

/// Creates the build list with advanced occlusion.
///
/// The chunks are traversed using breadth-first search and by consulting
/// their visibility graphs so that only the visible faces of the chunks are
/// rendered.
pub fn olad_build_draw_list(
    lookup: &HashMap<ChunkPos, ChunkDrawInfo>,
    player: &Player,
) -> Vec<DrawIndirectArgs> {
    let mut indirect_data = vec![];
    let frustum_planes = calculate_frustum_planes(player);

    let mut search_queue = VecDeque::new();
    let mut visited = HashSet::new();

    // draw all sides of the chunk that the player is in
    {
        let draw_info = &lookup
            .get(&player.get_chunk_pos())
            .expect("player should be in loaded world");
        Side::iter().for_each(|side| {
            indirect_data.push(create_chunk_indirect_args(draw_info, *side));
        });
    }

    // the start of the search queue
    let neighbors = get_neighbors(lookup, player.get_chunk_pos());
    for (chunk_pos, side) in neighbors {
        if !is_chunk_inside_frustum(chunk_pos, frustum_planes) {
            continue;
        }

        // draw and push to search
        let draw_info = &lookup.get(&chunk_pos).expect("should be loaded");
        indirect_data.push(create_chunk_indirect_args(draw_info, side));

        search_queue.push_back((chunk_pos, side));
    }

    while !search_queue.is_empty() {
        let (current_pos, parent_side) = search_queue.pop_front().expect("should not be empty");

        if visited.contains(&current_pos) {
            continue;
        }
        visited.insert(current_pos);

        // get the graph for the current chunk
        let graph = &lookup
            .get(&current_pos)
            .expect("should exist in lookup")
            .vis_graph;

        let neighbors = get_neighbors(lookup, current_pos);
        for (next_chunk, side) in neighbors {
            // apply the filters, might want to break the control flow apart so
            // no redundant computation is done
            let already_seen = visited.contains(&next_chunk);
            let in_frustum = is_chunk_inside_frustum(next_chunk, frustum_planes);
            let can_see_from_parent = graph.can_reach_from(parent_side, side);

            if already_seen || !in_frustum || !can_see_from_parent {
                continue;
            }

            // if it passes the filters, then render it
            let draw_info = lookup.get(&next_chunk).expect("should be in lookup");
            indirect_data.push(create_chunk_indirect_args(draw_info, side));

            search_queue.push_back((next_chunk, side.opposite()));
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

/// Returns a vector of loaded chunks neighboring the passed in chunk_pos.
pub fn get_neighbors(
    lookup: &HashMap<ChunkPos, ChunkDrawInfo>,
    pos: ChunkPos,
) -> Vec<(ChunkPos, Side)> {
    let mut output = Vec::new();

    let top = ChunkPos(pos.0, pos.1 + 1, pos.2);
    if lookup.contains_key(&top) {
        output.push((top, Side::TOP));
    }

    let bottom = ChunkPos(pos.0, pos.1 - 1, pos.2);
    if lookup.contains_key(&bottom) {
        output.push((bottom, Side::BOTTOM));
    }

    let left = ChunkPos(pos.0 - 1, pos.1, pos.2);
    if lookup.contains_key(&left) {
        output.push((left, Side::LEFT));
    }

    let right = ChunkPos(pos.0 + 1, pos.1, pos.2);
    if lookup.contains_key(&right) {
        output.push((right, Side::RIGHT));
    }

    let front = ChunkPos(pos.0, pos.1, pos.2 - 1);
    if lookup.contains_key(&front) {
        output.push((front, Side::FRONT));
    }

    let back = ChunkPos(pos.0, pos.1, pos.2 + 1);
    if lookup.contains_key(&back) {
        output.push((back, Side::BACK));
    }

    output
}
