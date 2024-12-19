use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{chunk::pool::ChunkPool, player::Player};

use super::window_state::WindowState;

#[derive(Default)]
pub struct Game {
    window: Option<WindowState>,
    player: Player,
    chunk_pool: ChunkPool,
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Set up window
        let (width, height) = (800, 500);
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_title("Voxel Visualization");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        log::info!("Window created.");

        let w = pollster::block_on(WindowState::new(window));

        // set up game objects

        self.chunk_pool = ChunkPool::initialize(&w);
        self.player = Player {};

        self.window = Some(w);
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
                    self.chunk_pool.update_chunks(&self.player);
                }

                self.chunk_pool.render(state, &self.player);

                state.window.request_redraw();
            }
            _ => {}
        }
    }
}
