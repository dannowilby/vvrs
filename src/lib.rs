use game::Game;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

pub mod block;
pub mod chunk;
pub mod game;
pub mod window_state;

pub fn run() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut game = Game::default();
    event_loop.run_app(&mut game)
}
