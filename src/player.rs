use crate::chunk::CHUNK_SIZE;

#[derive(Default)]
pub struct Player {
    pub position: (i32, i32, i32),
    pub load_radius: u32,
}

impl Player {
    pub fn has_changed_chunk(&self) -> bool {
        false
    }

    pub fn get_chunk_pos(&self) -> (i32, i32, i32) {
        let mut x = self.position.0 / CHUNK_SIZE as i32;
        if self.position.0 < 0 {
            x -= 1;
        }

        let mut y = self.position.1 / CHUNK_SIZE as i32;
        if self.position.1 < 0 {
            y -= 1;
        }

        let mut z = self.position.2 / CHUNK_SIZE as i32;
        if self.position.2 < 0 {
            z -= 1;
        }

        (x, y, z)
    }
}
