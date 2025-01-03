use core::f32;

use cgmath::{InnerSpace, Matrix4, SquareMatrix};
use winit::keyboard::KeyCode;

use crate::{chunk::CHUNK_SIZE, input::Input};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Player {
    pub position: (i32, i32, i32),
    pub load_radius: u32,

    speed: f32,

    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,

    pub projection: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: (0, 0, 0),
            load_radius: 0,

            speed: 1.0,

            eye: cgmath::Point3::<f32> {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            target: cgmath::Point3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            up: cgmath::Vector3::<f32> {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },

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

    pub fn update_camera(&mut self, input: &Input, delta: f32) {
        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if input.get_key(KeyCode::KeyW) > 0.0 && forward_mag > self.speed {
            self.eye += forward_norm * self.speed * delta;
        }

        if input.get_key(KeyCode::KeyS) > 0.0 {
            self.eye -= forward_norm * self.speed * delta;
        }

        let right = forward_norm.cross(self.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = self.target - self.eye;
        let forward_mag = forward.magnitude();

        if input.get_key(KeyCode::KeyD) > 0.0 {
            self.eye =
                self.target - (forward + right * self.speed).normalize() * forward_mag * delta;
        }
        if input.get_key(KeyCode::KeyS) > 0.0 {
            self.eye =
                self.target - (forward - right * self.speed).normalize() * forward_mag * delta;
        }

        self.view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
    }

    pub fn get_projection(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * self.projection
    }

    pub fn get_view(&self) -> cgmath::Matrix4<f32> {
        self.view
    }
}
