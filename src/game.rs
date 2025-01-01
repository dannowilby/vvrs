use std::{sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{chunk::manager::ChunkManager, player::Player};

use super::window_state::WindowState;

/// TODO:
/// - Encode the vertices properly
/// - Add camera input
/// - Implement resizing
/// - Add frustum culling
/// - Visibility graphs?
#[derive(Default)]
pub struct Game {
    window: Option<WindowState>,
    player: Player,
    chunk_m: ChunkManager,

    acc_time: f32,
    frames: u32,
    time: Option<Instant>,
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let now = Instant::now();

        // Set up window
        let (width, height) = (800, 500);
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_title("Voxel Visualization");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        log::info!("Window created.");

        let w = pollster::block_on(WindowState::new(window));

        // set up game objects, player is set up by Default

        self.chunk_m.init(&w);

        self.chunk_m.load_chunks(&w, &self.player);

        self.window = Some(w);

        self.time = Some(Instant::now());

        log::info!(
            "Initial loading took {}s",
            now.elapsed().as_micros() as f32 / 1_000_000.0
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.window else {
            return;
        };

        // 16000us = 16ms
        let delta: f32 = self.time.expect("").elapsed().as_micros() as f32 / 1000.0;
        self.acc_time += delta; // delta in milliseconds
        self.frames += 1; // add a frame
        if self.acc_time > 1000.0 {
            // if more than 1000 milliseconds
            log::info!("Avg FPS in prev second: {:.2}", self.frames as f32);
            self.acc_time -= 1000.0;
            self.frames = 0;
        }
        self.time = Some(Instant::now());

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                state.surface_config.width = size.width;
                state.surface_config.height = size.height;
                state
                    .surface
                    .configure(&state.device, &state.surface_config);

                state.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.player.has_changed_chunk() {
                    self.chunk_m.load_chunks(state, &self.player);
                }

                self.chunk_m.render(state, &self.player);

                state.window.request_redraw();
            }
            _ => {}
        }
    }
}
