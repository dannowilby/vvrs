/// since this is a tuple struct, it has the same memory as just a u32,
/// or so I believe
#[derive(Clone, Copy)]
pub struct Block(pub u32);

impl Block {
    pub fn is_solid(&self) -> bool {
        match self.0 {
            0 => false,
            1 => true,
            _ => false,
        }
    }
}
