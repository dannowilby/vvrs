use std::{sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{Window, WindowId},
};

use crate::{chunk::manager::ChunkManager, input::Input, player::Player};

use super::window_state::WindowState;

/// TODO:
/// - Fix camera controller
/// - Fix face encoding
/// - Add frustum culling
/// - Visibility graphs?
#[derive(Default)]
pub struct Game {
    window: Option<WindowState>,
    player: Player,
    chunk_m: ChunkManager,

    input: Input,

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

        // for some reason, events are only captured by this if not nested in
        // the following match statement
        self.input.handle(state, &event);

        match &event {
            WindowEvent::Resized(size) => {
                state.surface_config.width = size.width;
                state.surface_config.height = size.height;
                state
                    .surface
                    .configure(&state.device, &state.surface_config);
                self.player.resize(size.width as f32 / size.height as f32);

                state.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // delta calculations (in seconds/floating point fraction)
                let delta: f32 =
                    self.time.expect("").elapsed().as_micros() as f32 / (1000.0 * 1000.0);

                self.acc_time += delta;
                self.frames += 1;
                if self.acc_time > 1.0 {
                    // if more than 1 second
                    log::info!("FPS: {:.2}", self.frames as f32);
                    log::info!("Current frame delta: {}", self.input.is_focused);
                    self.acc_time -= 1.0;
                    self.frames = 0;
                }
                self.time = Some(Instant::now());

                if self.input.get_key(KeyCode::Escape) > 0.0 {
                    event_loop.exit();
                }

                // actual game logic
                self.player.update_camera(&mut self.input, delta);

                if self.player.has_changed_chunk() {
                    self.chunk_m.load_chunks(state, &self.player);
                }

                self.chunk_m.render(state, &self.player);

                state.window.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.input.is_focused {
                log::info!("{:?}", delta);
                self.input.mouse_delta(delta);
            }
        }
    }
}
