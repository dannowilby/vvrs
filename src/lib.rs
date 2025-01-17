use game::Game;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

pub mod chunk;
pub mod game;
pub mod input;
pub mod player;
pub mod util;
pub mod window_state;

pub fn run() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut game = Game::default();
    event_loop.run_app(&mut game)
}
