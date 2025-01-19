use std::collections::VecDeque;
use std::slice::Iter;

use crate::util::vec_set::VecSet;

use super::{Chunk, LocalBlockPos, CHUNK_SIZE};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Side {
    FRONT = 0,
    BACK = 1,
    TOP = 2,
    BOTTOM = 3,
    LEFT = 4,
    RIGHT = 5,
}

impl Side {
    #[allow(dead_code)]
    pub fn normal(&self) -> cgmath::Vector3<f32> {
        match self {
            Side::FRONT => cgmath::Vector3::new(0.0, 0.0, 1.0),
            Side::BACK => cgmath::Vector3::new(0.0, 0.0, -1.0),
            Side::TOP => cgmath::Vector3::new(0.0, 1.0, 0.0),
            Side::BOTTOM => cgmath::Vector3::new(0.0, -1.0, 0.0),
            Side::LEFT => cgmath::Vector3::new(-1.0, 0.0, 0.0),
            Side::RIGHT => cgmath::Vector3::new(1.0, 0.0, 0.0),
        }
    }
    pub fn opposite(self) -> Self {
        match self {
            Side::FRONT => Side::BACK,
            Side::BACK => Side::FRONT,
            Side::TOP => Side::BOTTOM,
            Side::BOTTOM => Side::TOP,
            Side::LEFT => Side::RIGHT,
            Side::RIGHT => Side::LEFT,
        }
    }
    pub fn iter() -> Iter<'static, Side> {
        static SIDES: [Side; 6] = [
            Side::FRONT,
            Side::BACK,
            Side::TOP,
            Side::BOTTOM,
            Side::LEFT,
            Side::RIGHT,
        ];

        SIDES.iter()
    }
}

/// A 6x6 matrix to keep track of which sides we can enter and exit from.
#[derive(Debug)]
pub struct VisibilityGraph([[bool; 6]; 6]);

impl VisibilityGraph {
    pub const EMPTY_GRAPH: VisibilityGraph = VisibilityGraph([
        [true, false, false, false, false, false],
        [false, true, false, false, false, false],
        [false, false, true, false, false, false],
        [false, false, false, true, false, false],
        [false, false, false, false, true, false],
        [false, false, false, false, false, true],
    ]);

    // flood fill in this function
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let mut connections = Vec::<(Side, Side)>::new();
        let depth = CHUNK_SIZE;

        // get a vector of blocks we can flood fill on
        let mut fill_seeds = VecSet::new();
        for x in 0..depth {
            for y in 0..depth {
                for z in 0..depth {
                    // only do flood fill if we are on the edge of the chunk
                    if x != 0
                        && x - 1 != depth
                        && y != 0
                        && y - 1 != depth
                        && z != 0
                        && z - 1 != depth
                    {
                        continue;
                    }

                    let block = chunk.data.get(&LocalBlockPos(x, y, z));

                    // start at an empty block
                    if block.is_none() || !block.expect("Block is not none!").is_solid() {
                        fill_seeds.insert(LocalBlockPos(x, y, z));
                    }
                }
            }
        }

        let mut visited = VecSet::new();
        while !fill_seeds.is_empty() {
            let pos = fill_seeds.remove_front().unwrap();
            // flood fill to get sides we can exit out of
            let sides = flood_fill(chunk, &pos, &mut fill_seeds, &mut visited);

            // create tuples of sides that can reach each other from the result of flood fill
            // add the tuples to the connections vec
            for i in 0..(sides.len() - 1) {
                for j in i..sides.len() {
                    let connection1 = (sides[i], sides[j]);
                    let connection2 = (sides[j], sides[i]);
                    if !connections.contains(&connection1) {
                        connections.push(connection1);
                    }
                    if !connections.contains(&connection2) {
                        connections.push(connection2);
                    }
                }
            }
        }

        // start the floodfill
        // everytime we hit a block in our vector, we remove it

        // once we have our connections vec,
        // modify the graph to reflect the connections we found
        let mut output = VisibilityGraph::EMPTY_GRAPH;
        for (side1, side2) in connections.into_iter() {
            let x = side1 as usize;
            let y = side2 as usize;
            output.0[x][y] = true;
        }

        output
    }

    pub fn can_reach_from(&self, side1: Side, side2: Side) -> bool {
        let matrix = self.0;

        let x = side1 as usize;
        let y = side2 as usize;

        matrix[x][y]
    }
}

fn flood_fill(
    chunk: &Chunk,
    start_pos: &LocalBlockPos,
    fill_seeds: &mut VecSet<LocalBlockPos>,
    visited: &mut VecSet<LocalBlockPos>,
) -> Vec<Side> {
    // might be more semantic to return a set as we want the values in the vec to be unique
    let mut output = Vec::new();
    let mut stack = VecDeque::<LocalBlockPos>::from([*start_pos]);

    while !stack.is_empty() {
        let pos = stack.pop_front().unwrap();

        // remove it from the possible seeds in the future
        // this will help cut down on duplicate fills
        fill_seeds.remove(&pos);

        // if we've already visited, then continue
        if visited.contains(&pos) {
            continue;
        }
        visited.insert(pos);

        // skip if pos isn't in the dimensions of the chunk
        if pos.0 + 1 > CHUNK_SIZE || pos.1 + 1 > CHUNK_SIZE || pos.2 + 1 > CHUNK_SIZE {
            continue;
        }

        // check if air
        // if not continue
        let block = chunk.data.get(&pos);
        if block.is_some() && block.expect("Block is some.").is_solid() {
            continue;
        }

        // if it is air/transparent
        // check if it is on the side
        // if on the side, add the side to the output if not already added
        // and add its neighbors to the queue
        if pos.0 == 0 && !output.contains(&Side::RIGHT) {
            output.push(Side::RIGHT);
        }
        if pos.0 == CHUNK_SIZE - 1 && !output.contains(&Side::LEFT) {
            output.push(Side::LEFT);
        }

        if pos.1 == 0 && !output.contains(&Side::BOTTOM) {
            output.push(Side::BOTTOM);
        }
        if pos.1 == CHUNK_SIZE - 1 && !output.contains(&Side::TOP) {
            output.push(Side::TOP);
        }

        if pos.2 == 0 && !output.contains(&Side::FRONT) {
            output.push(Side::FRONT);
        }
        if pos.2 == CHUNK_SIZE - 1 && !output.contains(&Side::BACK) {
            output.push(Side::BACK);
        }

        if let Some(np) = LocalBlockPos::safe_add(&pos, &LocalBlockPos(1, 0, 0)) {
            stack.push_back(np);
        }
        if let Some(np) = LocalBlockPos::safe_sub(&pos, &LocalBlockPos(1, 0, 0)) {
            stack.push_back(np);
        }
        if let Some(np) = LocalBlockPos::safe_add(&pos, &LocalBlockPos(0, 1, 0)) {
            stack.push_back(np);
        }
        if let Some(np) = LocalBlockPos::safe_sub(&pos, &LocalBlockPos(0, 1, 0)) {
            stack.push_back(np);
        }
        if let Some(np) = LocalBlockPos::safe_add(&pos, &LocalBlockPos(0, 0, 1)) {
            stack.push_back(np);
        }
        if let Some(np) = LocalBlockPos::safe_sub(&pos, &LocalBlockPos(0, 0, 1)) {
            stack.push_back(np);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::chunk::block::Block;

    use super::*;

    #[test]
    fn visibility_graph_full_chunk() {
        let full_chunk = Chunk::full();

        let vis_graph = VisibilityGraph::from_chunk(&full_chunk);

        // iter over each entry to check if any are false
        for side1 in Side::iter() {
            for side2 in Side::iter() {
                if side1 == side2 {
                    assert!(vis_graph.can_reach_from(*side1, *side2));
                    continue;
                }
                assert!(!vis_graph.can_reach_from(*side1, *side2));
            }
        }
    }

    #[test]
    fn visibility_graph_empty_chunk() {
        let empty_chunk = Chunk::default();
        let vis_graph = VisibilityGraph::from_chunk(&empty_chunk);

        // iter over each entry to check if any are false
        for side1 in Side::iter() {
            for side2 in Side::iter() {
                assert!(vis_graph.can_reach_from(*side1, *side2));
            }
        }
    }

    #[test]
    fn visibility_graph_split_x_chunk() {
        let mut split_chunk = Chunk::default();

        // fill the chunk with blocks
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // 2 is just some random transparent that we want to check
                    if x == 3 {
                        split_chunk.set_block(LocalBlockPos(x, y, z), Block(1));
                    }
                }
            }
        }

        let vis_graph = VisibilityGraph::from_chunk(&split_chunk);

        for side1 in Side::iter() {
            for side2 in Side::iter() {
                if (*side1 == Side::RIGHT && *side2 == Side::LEFT)
                    || (*side2 == Side::RIGHT && *side1 == Side::LEFT)
                {
                    assert!(!vis_graph.can_reach_from(*side1, *side2));
                    continue;
                }

                assert!(
                    vis_graph.can_reach_from(*side1, *side2),
                    "{:?} - {:?}",
                    side1,
                    side2
                );
            }
        }
    }

    #[test]
    fn visibility_graph_split_y_chunk() {
        let mut split_chunk = Chunk::default();

        // fill the chunk with blocks
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // 2 is just some random transparent that we want to check
                    if y == 3 {
                        split_chunk.set_block(LocalBlockPos(x, y, z), Block(1));
                    }
                }
            }
        }

        let vis_graph = VisibilityGraph::from_chunk(&split_chunk);

        for side1 in Side::iter() {
            for side2 in Side::iter() {
                if (*side1 == Side::TOP && *side2 == Side::BOTTOM)
                    || (*side2 == Side::TOP && *side1 == Side::BOTTOM)
                {
                    assert!(!vis_graph.can_reach_from(*side1, *side2));
                    continue;
                }

                assert!(
                    vis_graph.can_reach_from(*side1, *side2),
                    "{:?} - {:?}",
                    side1,
                    side2
                );
            }
        }
    }

    #[test]
    fn visibility_graph_split_z_chunk() {
        let mut split_chunk = Chunk::default();

        // fill the chunk with blocks
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // 2 is just some random transparent that we want to check
                    if z == 3 {
                        split_chunk.set_block(LocalBlockPos(x, y, z), Block(1));
                    }
                }
            }
        }

        let vis_graph = VisibilityGraph::from_chunk(&split_chunk);

        for side1 in Side::iter() {
            for side2 in Side::iter() {
                if (*side1 == Side::FRONT && *side2 == Side::BACK)
                    || (*side2 == Side::FRONT && *side1 == Side::BACK)
                {
                    assert!(!vis_graph.can_reach_from(*side1, *side2));
                    continue;
                }

                assert!(
                    vis_graph.can_reach_from(*side1, *side2),
                    "{:?} - {:?}",
                    side1,
                    side2
                );
            }
        }
    }
}
