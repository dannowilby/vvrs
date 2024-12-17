use std::collections::HashMap;

use crate::block::Block;

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

/// Returns the mesh of the chunk. The resulting chunk is split by the direction
/// of the faces.
/// The greedy face merging is a fairly naive implmenetation and doesn't use
/// binary operations on the face mask. Doesn't seem like it will be a
/// bottleneck yet, but it can always be changed.
#[allow(dead_code)]
pub fn mesh(chunk: &Chunk) -> [Vec<u32>; 6] {
    // first we want create a binary representation of only the solid blocks,
    // so we can cull the non-visible faces that don't touch air
    let mut t = [[0u32; CHUNK_SIZE]; CHUNK_SIZE];

    for ((x, y, z), block) in chunk.data.iter() {
        if block.is_solid() {
            t[*x as usize][*y as usize] |= 1 << z;
        }
    }

    // for each axis (direction), we want to create a map of the faces,
    // ! with the block type !
    // also hashmaps don't allocate until the first insert, so allocating them
    // is fine
    let mut data = [
        HashMap::<(u32, u32, u32), Block>::new(),
        HashMap::<(u32, u32, u32), Block>::new(),
        HashMap::<(u32, u32, u32), Block>::new(),
        HashMap::<(u32, u32, u32), Block>::new(),
        HashMap::<(u32, u32, u32), Block>::new(),
        HashMap::<(u32, u32, u32), Block>::new(),
    ];

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            // the next next_row and previous_row default to zero to make the
            // faces show on the edges, if we add padding, just disregard them

            // cull z faces
            let z_quads_forward = t[x][y] & !(t[x][y] << 1);
            add_faces(chunk, &mut data[0], x, y, z_quads_forward);

            let z_quads_backward = t[x][y] & !(t[x][y] >> 1);
            add_faces(chunk, &mut data[3], x, y, z_quads_backward);

            // cull y faces
            let next_row = if y + 1 >= CHUNK_SIZE { 0 } else { t[x][y + 1] };
            let y_quads_forward = t[x][y] & !next_row;
            add_faces(chunk, &mut data[1], x, y, y_quads_forward);

            let previous_row = if y as i32 - 1 < 0 { 0 } else { t[x][y - 1] };
            let y_quads_backward = t[x][y] & !previous_row;
            add_faces(chunk, &mut data[4], x, y, y_quads_backward);

            // cull x faces
            let next_row = if x + 1 >= CHUNK_SIZE { 0 } else { t[x + 1][y] };
            let x_quads_forward = t[x][y] & !next_row;
            add_faces(chunk, &mut data[2], x, y, x_quads_forward);

            let previous_row = if x as i32 - 1 < 0 { 0 } else { t[x - 1][y] };
            let x_quads_backward = t[x][y] & !previous_row;
            add_faces(chunk, &mut data[5], x, y, x_quads_backward);
        }
    }

    // the vertex data itself
    let mut mesh = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    for i in 0..6 {
        mesh[i] = greedy_merge(&mut data[i], i);
    }

    mesh
}

/// Decodes the visible faces from the culling step
fn add_faces(
    chunk: &Chunk,
    data: &mut HashMap<(u32, u32, u32), Block>,
    x: usize,
    y: usize,
    faces: u32,
) {
    let mut faces = faces;

    let mut z = 0;

    while faces != 0 {
        let leading = faces.leading_zeros();

        // shifting the bits can cause an overflow if we're not careful
        // about how we do it
        faces <<= leading; // shift over the 0s
        faces -= 0x8000_0000; // subtract the most significant bit
        faces <<= 1; // shift it over now that it's 0

        z += leading + 1;

        data.insert(
            (x as u32, y as u32, CHUNK_SIZE as u32 - z),
            chunk.get_block(x as u32, y as u32, CHUNK_SIZE as u32 - z),
        );
    }
}

/// Greedy mesh the quads,
/// note: this is not guaranteed to produce optimal meshes
fn greedy_merge(hm: &mut HashMap<(u32, u32, u32), Block>, axis: usize) -> Vec<u32> {
    // create output mesh data vec
    let mut output = Vec::<u32>::new();

    let growth_axes = match axis as u32 % 3 {
        0 => [(1, 0, 0), (0, 1, 0)], // xy-plane
        1 => [(1, 0, 0), (0, 0, 1)], // xz-plane
        2 => [(0, 1, 0), (0, 0, 1)], // yz-plane
        _ => [(1, 1, 1), (1, 1, 1)],
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

        // check one block forward in the row
        while let Some(b) = hm.get(&(quad2.0 + i.0, quad2.1 + i.1, quad2.2 + i.2)) {
            if b == &block {
                hm.remove(&(quad2.0 + i.0, quad2.1 + i.1, quad2.2 + i.2));
                quad2 = (quad2.0 + i.0, quad2.1 + i.1, quad2.2 + i.2);
            } else {
                break;
            }
        }

        // check the blocks backward in the row
        while let Some(t) = safe_subtract_position(quad1, i) {
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

        // check one column backward
        let mut can_grow = true;
        while can_grow {
            let Some(t) = safe_subtract_position(quad1, j) else {
                break;
            };

            let mut to_remove = vec![];
            for l in 0..column_length {
                let a: (u32, u32, u32) = (t.0 + l * i.0, t.1 + l * i.1, t.2 + l * i.2);
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
            let t = (quad2.0 + j.0, quad2.1 + j.1, quad2.2 + j.2);

            let mut to_remove = vec![];
            for l in 0..column_length {
                let Some(a) = safe_subtract_position(t, (l * i.0, l * i.1, l * i.2)) else {
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

/// Encode the vertices, not fully implemented yet
fn create_quad(
    _axis: usize,
    (c1x, c1y, c1z): (u32, u32, u32),
    (c2x, c2y, c2z): (u32, u32, u32),
) -> Vec<u32> {
    vec![c1x, c1y, c1z, c2x, c2y, c2z]
}

fn safe_subtract_position(p1: (u32, u32, u32), p2: (u32, u32, u32)) -> Option<(u32, u32, u32)> {
    let x = p1.0.checked_sub(p2.0);
    let y = p1.1.checked_sub(p2.1);
    let z = p1.2.checked_sub(p2.2);

    if x.is_none() || y.is_none() || z.is_none() {
        return None;
    }

    Some((x.unwrap(), y.unwrap(), z.unwrap()))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn can_modify_chunk() {
        let mut chunk = Chunk::default();

        chunk.set_block(0, 0, 0, Block(1));

        let x = chunk.get_block(0, 0, 0);
        let y = chunk.get_block(1, 1, 1);

        assert!(x == Block(1));
        assert!(y == Block(0));
    }
}
