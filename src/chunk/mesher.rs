use std::{collections::HashMap, time::Instant};

use crate::chunk::{block::Block, ChunkDimTy, LocalBlockPos, CHUNK_SIZE};

use super::{Chunk, EncodedVertex, NUM_BITS_IN_POS};

/// Returns the mesh of the chunk. The resulting chunk is split by the direction
/// of the faces.
/// The greedy face merging is a fairly naive implmenetation and doesn't use
/// binary operations on the face mask. Doesn't seem like it will be a
/// bottleneck yet, but it can always be changed.
pub fn mesh(chunk: &Chunk) -> [Vec<EncodedVertex>; 6] {
    let cull_time = Instant::now();

    // first we want create a binary representation of only the solid blocks,
    // so we can cull the non-visible faces that don't touch air
    let mut t = [[ChunkDimTy::default(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

    for (LocalBlockPos(x, y, z), block) in chunk.data.iter() {
        if block.is_solid() {
            t[*x as usize][*y as usize] |= 1 << z;
        }
    }

    // for each axis (direction), we want to create a map of the faces,
    // ! with the block type !
    // also hashmaps don't allocate until the first insert, so allocating them
    // is fine
    let mut data = [
        HashMap::<LocalBlockPos, Block>::new(),
        HashMap::<LocalBlockPos, Block>::new(),
        HashMap::<LocalBlockPos, Block>::new(),
        HashMap::<LocalBlockPos, Block>::new(),
        HashMap::<LocalBlockPos, Block>::new(),
        HashMap::<LocalBlockPos, Block>::new(),
    ];

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            // the next next_row and previous_row default to zero to make the
            // faces show on the edges, if we add padding, just disregard them
            let x = x as usize;
            let y = y as usize;
            // cull z faces
            let z_quads_forward = t[x][y] & !(t[x][y] << 1);
            add_faces(chunk, &mut data[0], x, y, z_quads_forward);

            let z_quads_backward = t[x][y] & !(t[x][y] >> 1);
            add_faces(chunk, &mut data[3], x, y, z_quads_backward);

            // cull y faces
            let next_row = if y + 1 >= CHUNK_SIZE as usize {
                0
            } else {
                t[x][y + 1]
            };
            let y_quads_forward = t[x][y] & !next_row;
            add_faces(chunk, &mut data[1], x, y, y_quads_forward);

            let previous_row = if y as i32 - 1 < 0 { 0 } else { t[x][y - 1] };
            let y_quads_backward = t[x][y] & !previous_row;
            add_faces(chunk, &mut data[4], x, y, y_quads_backward);

            // cull x faces
            let next_row = if x + 1 >= CHUNK_SIZE as usize {
                0
            } else {
                t[x + 1][y]
            };
            let x_quads_forward = t[x][y] & !next_row;
            add_faces(chunk, &mut data[2], x, y, x_quads_forward);

            let previous_row = if x as i32 - 1 < 0 { 0 } else { t[x - 1][y] };
            let x_quads_backward = t[x][y] & !previous_row;
            add_faces(chunk, &mut data[5], x, y, x_quads_backward);
        }
    }
    log::debug!("Culling quads took {}us", cull_time.elapsed().as_micros());

    // the vertex data itself
    let mut mesh = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    let mesh_time = Instant::now();
    for i in 0..6 {
        mesh[i] = greedy_merge(&mut data[i], i);
    }
    log::debug!("Merging quads took {}us", mesh_time.elapsed().as_micros());

    mesh
}

/// Decodes the visible faces from the culling step
fn add_faces(
    chunk: &Chunk,
    data: &mut HashMap<LocalBlockPos, Block>,
    x: usize,
    y: usize,
    faces: ChunkDimTy,
) {
    let mut faces = faces;

    let mut z = 0;

    while faces != 0 {
        let leading = faces.leading_zeros() as ChunkDimTy; // why does this always return a u32?

        // shifting the bits can cause an overflow if we're not careful
        // about how we do it
        faces <<= leading; // shift over the 0s
        faces -= 1 << (CHUNK_SIZE - 1); // subtract the most significant bit
        faces <<= 1; // shift it over now that it's 0

        z += leading + 1;

        data.insert(
            LocalBlockPos(
                x as ChunkDimTy,
                y as ChunkDimTy,
                CHUNK_SIZE as ChunkDimTy - z,
            ),
            chunk.get_block(&LocalBlockPos(
                x as ChunkDimTy,
                y as ChunkDimTy,
                CHUNK_SIZE as ChunkDimTy - z,
            )),
        );
    }
}

/// Greedy mesh the quads,
/// note: this is not guaranteed to produce optimal meshes
/// THIS ALGORITHM HAS A BUG IN IT FFS
fn greedy_merge(hm: &mut HashMap<LocalBlockPos, Block>, axis: usize) -> Vec<EncodedVertex> {
    // create output mesh data vec
    let mut output = Vec::<EncodedVertex>::new();

    let growth_axes = match axis as u32 % 3 {
        0 => [LocalBlockPos(1, 0, 0), LocalBlockPos(0, 1, 0)], // xy-plane
        1 => [LocalBlockPos(1, 0, 0), LocalBlockPos(0, 0, 1)], // xz-plane
        2 => [LocalBlockPos(0, 1, 0), LocalBlockPos(0, 0, 1)], // yz-plane
        _ => [LocalBlockPos(1, 1, 1), LocalBlockPos(1, 1, 1)],
    };

    while !hm.is_empty() {
        // get an element
        let (pos, block) = hm.iter().take(1).collect::<Vec<_>>()[0];
        let pos = *pos; // we clone the values to avoid appease the borrow checker
        let block = *block;

        hm.remove(&pos);

        let i = growth_axes[0];
        let j = growth_axes[1];

        let mut quad1 = pos;
        let mut quad2 = pos;

        // check block forward in the row
        while let Some(b) = hm.get(&LocalBlockPos(quad2.0 + i.0, quad2.1 + i.1, quad2.2 + i.2)) {
            if b == &block {
                hm.remove(&LocalBlockPos(quad2.0 + i.0, quad2.1 + i.1, quad2.2 + i.2));
                quad2 = LocalBlockPos(quad2.0 + i.0, quad2.1 + i.1, quad2.2 + i.2);
            } else {
                break;
            }
        }

        // check the blocks backward in the row
        while let Some(t) = LocalBlockPos::safe_sub(&quad1, &i) {
            if let Some(b) = hm.get(&t) {
                if b == &block {
                    hm.remove(&t);
                    quad1 = t;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let column_length =
            1 + (quad2.0 - quad1.0) * i.0 + (quad2.1 - quad1.1) * i.1 + (quad2.2 - quad1.2) * i.2;

        // check column backward
        let mut can_grow = true;
        while can_grow {
            let Some(t) = LocalBlockPos::safe_sub(&quad1, &j) else {
                break;
            };

            let mut to_remove = vec![];
            for l in 0..column_length {
                let a = LocalBlockPos(t.0 + l * i.0, t.1 + l * i.1, t.2 + l * i.2);
                let c = hm.get(&a);

                if let Some(b) = c {
                    if b == &block {
                        to_remove.push(a);
                        continue;
                    }
                }

                can_grow = false;
            }

            if can_grow {
                quad1 = t;
                for k in to_remove {
                    hm.remove(&k);
                }
            }
        }

        // check column forward
        can_grow = true;
        while can_grow {
            let t = LocalBlockPos(quad2.0 + j.0, quad2.1 + j.1, quad2.2 + j.2);

            let mut to_remove = vec![];
            for l in 0..column_length {
                let Some(a) =
                    LocalBlockPos::safe_sub(&t, &LocalBlockPos(l * i.0, l * i.1, l * i.2))
                else {
                    can_grow = false;
                    break;
                };

                if let Some(b) = hm.get(&a) {
                    if b == &block {
                        to_remove.push(a);
                        continue;
                    }
                }

                can_grow = false;
            }

            if can_grow {
                quad2 = t;
                for k in to_remove {
                    hm.remove(&k);
                }
            }
        }
        output.append(&mut create_quad(axis, quad1, quad2));
    }

    output
}

/// Encode the vertices of a quad, defined by two opposite corners.
fn create_quad(
    axis: usize, // Axis along which the face is oriented: 0-5 for six cube faces
    LocalBlockPos(c1x, c1y, c1z): LocalBlockPos,
    LocalBlockPos(c2x, c2y, c2z): LocalBlockPos,
) -> Vec<EncodedVertex> {
    // Determine the min and max bounds of the corners

    // the positions are 0-31 inclusive, whereas
    // the vertices are 0-32 inclusive, so encoding them
    // with 5 bits causes issues at the upper ends of the blocks
    // we still need to add one to quad2,
    let min_x = c1x.min(c2x + 1);
    let max_x = c1x.max(c2x + 1);
    let min_y = c1y.min(c2y + 1);
    let max_y = c1y.max(c2y + 1);
    let min_z = c1z.min(c2z + 1);
    let max_z = c1z.max(c2z + 1);

    // Generate vertices based on the axis
    match axis {
        2 => vec![
            // +X face
            encode_vertex(max_x, min_y, min_z),
            encode_vertex(max_x, max_y, min_z),
            encode_vertex(max_x, max_y, max_z),
            encode_vertex(max_x, min_y, min_z),
            encode_vertex(max_x, max_y, max_z),
            encode_vertex(max_x, min_y, max_z),
        ],
        5 => vec![
            // -X face
            encode_vertex(min_x, min_y, min_z),
            encode_vertex(min_x, max_y, min_z),
            encode_vertex(min_x, max_y, max_z),
            encode_vertex(min_x, min_y, min_z),
            encode_vertex(min_x, max_y, max_z),
            encode_vertex(min_x, min_y, max_z),
        ],
        1 => vec![
            // +Y face
            encode_vertex(min_x, max_y, min_z),
            encode_vertex(max_x, max_y, min_z),
            encode_vertex(max_x, max_y, max_z),
            encode_vertex(min_x, max_y, min_z),
            encode_vertex(max_x, max_y, max_z),
            encode_vertex(min_x, max_y, max_z),
        ],
        4 => vec![
            // -Y face
            encode_vertex(min_x, min_y, min_z),
            encode_vertex(max_x, min_y, min_z),
            encode_vertex(max_x, min_y, max_z),
            encode_vertex(min_x, min_y, min_z),
            encode_vertex(max_x, min_y, max_z),
            encode_vertex(min_x, min_y, max_z),
        ],
        3 => vec![
            // +Z face
            encode_vertex(min_x, min_y, max_z),
            encode_vertex(max_x, min_y, max_z),
            encode_vertex(max_x, max_y, max_z),
            encode_vertex(min_x, min_y, max_z),
            encode_vertex(max_x, max_y, max_z),
            encode_vertex(min_x, max_y, max_z),
        ],
        0 => vec![
            // -Z face
            encode_vertex(min_x, min_y, min_z),
            encode_vertex(max_x, min_y, min_z),
            encode_vertex(max_x, max_y, min_z),
            encode_vertex(min_x, min_y, min_z),
            encode_vertex(max_x, max_y, min_z),
            encode_vertex(min_x, max_y, min_z),
        ],
        _ => panic!("Invalid axis value: must be 0-5"),
    }
}

/// Helper function to encode a vertex position into a single value.
fn encode_vertex(x: ChunkDimTy, y: ChunkDimTy, z: ChunkDimTy) -> EncodedVertex {
    let mut output = 0;
    output |= x;
    output <<= NUM_BITS_IN_POS;
    output |= y;
    output <<= NUM_BITS_IN_POS;
    output |= z;

    EncodedVertex(output)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[allow(dead_code)]
    fn decode_vertex(v: &EncodedVertex) -> (ChunkDimTy, ChunkDimTy, ChunkDimTy) {
        let mut t = v.0;

        let z = t & 63;
        t >>= NUM_BITS_IN_POS;
        let y = t & 63;
        t >>= NUM_BITS_IN_POS;
        let x = t & 63;

        (x, y, z)
    }

    #[test]
    fn can_modify_chunk() {
        let mut chunk = Chunk::default();

        chunk.set_block(LocalBlockPos(0, 0, 0), Block(1));

        let x = chunk.get_block(&LocalBlockPos(0, 0, 0));
        let y = chunk.get_block(&LocalBlockPos(1, 1, 1));

        assert!(x == Block(1));
        assert!(y == Block(0));
    }

    #[test]
    fn can_mesh_chunk() {
        let mut chunk = Chunk::default();

        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    let pos = LocalBlockPos(i, j, k);
                    chunk.set_block(pos, Block(1));
                }
            }
        }

        let data = mesh(&chunk);

        for i in data {
            println!(
                "{:?}",
                i.iter().map(|f| decode_vertex(f)).collect::<Vec<_>>()
            );
            assert!(i.len() == 6);
        }
    }
}
