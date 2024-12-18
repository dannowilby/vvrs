/// since this is a tuple struct, it has the same memory as just a u32,
/// or so I believe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block(pub u32);

impl Block {
    pub fn is_solid(&self) -> bool {
        match self.0 {
            1 => true,  // dirt
            2 => true,  // stone
            3 => false, // leaves
            _ => false,
        }
    }

    pub fn get_uv(&self) -> (f32, f32) {
        match self.0 {
            1 => (0.0, 0.0),
            2 => (0.125, 0.0),
            3 => (0.25, 0.0),
            _ => (0.0, 0.0),
        }
    }
}
