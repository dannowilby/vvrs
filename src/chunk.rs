use std::collections::HashMap;

use crate::block::Block;

pub const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    pub data: HashMap<(u32, u32, u32), Block>,
}

impl Chunk {
    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Block {
        *self.data.get(&(x, y, z)).unwrap_or(&Block(0))
    }
}


fn mesh(chunk: &Chunk) -> [Vec<u32>; 6] {

    // first we want create a binary representation of only the solid blocks,
    // so we can cull the non-visible faces

    let mut t = [[0u32; CHUNK_SIZE]; CHUNK_SIZE];

    for ((x, y, z), block) in chunk.data.iter() {
        if block.is_solid() {
            t[*x as usize][*y as usize] &= 1; // shift the bit over by z
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

    // now that we have the faces
    // we want to build the biggest quads we canW
    // then mesh the quads

    for axis in 0..3 {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {

                todo!("Replace hard-coded values with actual indices");
                
                match axis {
                    0 => {
        
                        let z_quads_forward = t[0][0] & (!t[0][0] << 1);

                        let z_quads_backward = t[0][0] & (!t[0][0] >> 1);
                        
                        todo!("Index block and add face to hashmap");
                    }
                    1 => {
                        
                        let y_quads_forward = t[0][0] & !t[0][1];

                        let y_quads_backward = t[0][0] & !t[0][-1];
        
                        todo!("Index block and add face to hashmap");
                    }
                    2 => {
                        
                        let x_quads_forward = t[0][0] & !t[1][0];

                        let x_quads_backward = t[0][0] & !t[1 + 1][0];

                        todo!("Index block and add face to hashmap");
                    }
                    _ => {}
                }

            }
        }
    }

    let mut mesh = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new()
    ];

    for hm in data {
        todo!("Replace hard-coded values");
        mesh[0] = greedy_merge(hm);
    }


    return mesh;

}

fn greedy_merge(hm: HashMap<(u32, u32, u32), u32>) -> Vec<u32> {
    vec![]
}