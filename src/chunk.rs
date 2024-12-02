use std::collections::HashMap;

pub struct Chunk {
    pub data: HashMap<(u32, u32, u32), u32>,
}
