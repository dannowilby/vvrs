use std::collections::HashMap;

use crate::block::Block;

pub const CHUNK_SIZE: usize = 32;

// May want to consider storing padding by default, which also means updating
// the meshing code for this (should be easy, would just need to disregard the
// most/least significant bits of the mask)
pub struct Chunk {
    pub data: HashMap<(u32, u32, u32), Block>,
}

impl Chunk {
    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Block {
        *self.data.get(&(x, y, z)).unwrap_or(&Block(0))
    }
}

/// Returns the mesh of the chunk. The resulting chunk is split by the direction
/// of the faces.
#[allow(dead_code)]
fn mesh(chunk: &Chunk) -> [Vec<u32>; 6] {
    // first we want create a binary representation of only the solid blocks,
    // so we can cull the non-visible faces that don't touch air

    let mut t = [[0u32; CHUNK_SIZE]; CHUNK_SIZE];

    for ((x, y, z), block) in chunk.data.iter() {
        if block.is_solid() {
            t[*x as usize][*y as usize] |= 1 << z;
        }
    }

    // now we want to separate the visible faces into their block types

    // for each axis (direction), we want to create a map of the faces,
    // ! with the block type !
    // also hashmaps don't allocate until the first insert
    let mut data = [
        HashMap::<(u32, u32, u32), u32>::new(),
        HashMap::<(u32, u32, u32), u32>::new(),
        HashMap::<(u32, u32, u32), u32>::new(),
        HashMap::<(u32, u32, u32), u32>::new(),
        HashMap::<(u32, u32, u32), u32>::new(),
        HashMap::<(u32, u32, u32), u32>::new(),
    ];

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            // the next next_row and previous_row default to zero to make the
            // faces show on the edges, if we add padding, just disregard them

            // cull z faces
            let z_quads_forward = t[x][y] & !(t[x][y] << 1);
            add_faces(chunk, &mut data[0], x, y, z_quads_forward);

            let z_quads_backward = t[x][y] & !(t[x][y] >> 1);
            add_faces(chunk, &mut data[1], x, y, z_quads_backward);

            // cull y faces
            let next_row = if y + 1 >= CHUNK_SIZE { 0 } else { t[x][y + 1] };
            let y_quads_forward = t[x][y] & !next_row;
            add_faces(chunk, &mut data[2], x, y, y_quads_forward);

            let previous_row = if y as i32 - 1 < 0 { 0 } else { t[x][y - 1] };
            let y_quads_backward = t[x][y] & !previous_row;
            add_faces(chunk, &mut data[3], x, y, y_quads_backward);

            // cull x faces
            let next_row = if x + 1 >= CHUNK_SIZE { 0 } else { t[x + 1][y] };
            let x_quads_forward = t[x][y] & !next_row;
            add_faces(chunk, &mut data[4], x, y, x_quads_forward);

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
    data: &mut HashMap<(u32, u32, u32), u32>,
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
            chunk.get_block(x as u32, y as u32, CHUNK_SIZE as u32 - z).0,
        );
    }
}

fn greedy_merge(_hm: &mut HashMap<(u32, u32, u32), u32>, _axis: usize) -> Vec<u32> {
    // create output mesh data vec
    // sort the quads into an ordering based on position

    // loop until sorted list is empty
    // get the least element
    // try merge with quads around it
    // remove quad and merged from sorted list
    // if can't loop any more
    // add quad vertices to mesh data

    // return output mesh data vec

    todo!("Implement pseudocode.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn mesh_one_block() {
        // let mut chunk = Chunk { data: HashMap::new() };
        // chunk.data.insert((0, 0, 0), Block(1));
        // chunk.data.insert((0, 1, 0), Block(1));
        // mesh(&chunk);
        assert!(true);
    }
}
