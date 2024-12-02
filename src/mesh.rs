use crate::chunk::Chunk;

pub fn mesh_chunk(chunk: &Chunk, lod: u32) -> () {
    todo!("Think about LOD type");
    todo!("Think about vertex data return type");
    todo!("Implement binary greedy meshing");
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;

    #[test]
    fn mesh_empty_chunk() {
        let chunk = Chunk {
            data: HashMap::new(),
        };
        let vertex_data = mesh_chunk(&chunk, 0);

        assert_eq!(vertex_data, ());
    }

    #[test]
    fn mesh_nonempty_chunk() {}

    #[test]
    fn mesh_non_full_lod() {}
}
