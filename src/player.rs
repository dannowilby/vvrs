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
    pub position: cgmath::Point3<f32>,
    pub load_radius: u32,

    speed: f32,
    sensitivity: f32,
    yaw: f32,
    pitch: f32,

    pub projection: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: cgmath::Point3::<f32>::new(0.0, 0.0, 0.0),
            load_radius: 0,

            speed: 1.0,
            sensitivity: 100.0,
            yaw: 0.0,
            pitch: 0.0,

            projection: cgmath::perspective(cgmath::Deg(45.0), 800.0 / 500.0, 0.1, 1000.0),
            view: Matrix4::identity(),
        }
    }
}

impl Player {
    pub fn has_changed_chunk(&self) -> bool {
        false
    }

    pub fn get_chunk_pos(&self) -> (i32, i32, i32) {
        let mut x = (self.position.x / CHUNK_SIZE as f32).floor() as i32;
        if self.position.x < 0.0 {
            x -= 1;
        }

        let mut y = (self.position.y / CHUNK_SIZE as f32).floor() as i32;
        if self.position.y < 0.0 {
            y -= 1;
        }

        let mut z = (self.position.z / CHUNK_SIZE as f32).floor() as i32;
        if self.position.z < 0.0 {
            z -= 1;
        }

        (x, y, z)
    }

    pub fn resize(&mut self, aspect: f32) {
        self.projection = cgmath::perspective(cgmath::Deg(45.0), aspect, 0.1, 1000.0);
    }

    pub fn update_camera(&mut self, input: &mut Input, delta: f32) {
        // don't update if not focused
        if !input.is_focused {
            return;
        }

        let mov = self.calculate_player_input_velocity(input, delta);
        self.position += mov;

        if input.movement.0 != 0.0 || input.movement.1 != 0.0 {
            self.yaw += (input.movement.0) as f32 * delta * self.sensitivity;
            self.pitch -= (input.movement.1) as f32 * delta * self.sensitivity;
            input.movement = (0.0, 0.0);
        }

        // 1.55 is just below 2pi
        self.pitch = self.pitch.clamp(-1.55, 1.55);

        let up = cgmath::Vector3::<f32>::new(0.0, 1.0, 0.0);

        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let facing =
            cgmath::Vector3::<f32>::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin)
                .normalize();

        self.view = cgmath::Matrix4::look_to_rh(self.position, facing, up);
    }

    pub fn get_projection(&self) -> cgmath::Matrix4<f32> {
        self.projection
    }

    pub fn get_view(&self) -> cgmath::Matrix4<f32> {
        self.view
    }

    /// Get the velocity from player input.
    fn calculate_player_input_velocity(&self, input: &Input, delta: f32) -> cgmath::Vector3<f32> {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = cgmath::Vector3::<f32>::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = cgmath::Vector3::<f32>::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let mut pos = cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0);
        let move_speed = self.speed;
        let s = input.get_key(KeyCode::KeyS);
        let w = input.get_key(KeyCode::KeyW);
        let a = input.get_key(KeyCode::KeyA);
        let d = input.get_key(KeyCode::KeyD);
        let space = input.get_key(KeyCode::Space);
        let shift = input.get_key(KeyCode::ShiftLeft);
        if s > 0.0 {
            pos -= forward * move_speed * delta;
        }
        if w > 0.0 {
            pos += forward * move_speed * delta;
        }
        if a > 0.0 {
            pos -= right * move_speed * delta;
        }
        if d > 0.0 {
            pos += right * move_speed * delta;
        }
        if space > 0.0 {
            pos.y += move_speed * delta;
        }
        if shift > 0.0 {
            pos.y -= move_speed * delta;
        }
        pos
    }
}
