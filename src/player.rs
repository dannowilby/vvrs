use core::f32;

use cgmath::{Matrix4, SquareMatrix};

use crate::chunk::CHUNK_SIZE;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[allow(dead_code)]
pub struct Player {
    pub position: (i32, i32, i32),
    pub load_radius: u32,

    projection: cgmath::Matrix4<f32>,
    view: cgmath::Matrix4<f32>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: (0, 0, 0),
            load_radius: 2,

            projection: cgmath::perspective(
                cgmath::Rad(f32::consts::PI * 0.70),
                800.0 / 500.0,
                0.1,
                100.0,
            ),
            view: Matrix4::identity(),
        }
    }
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
