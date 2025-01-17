//! Used to capture mouse and keyboard events.

use std::{collections::HashMap, time::Instant};

use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::window_state::WindowState;

/// Struct used to track how long a key or mouse button has been pressed. Also
/// stores mouse movement delta and whether or not the window is focused.
#[derive(Default)]
pub struct Input {
    pub is_focused: bool,
    pub keys: HashMap<KeyCode, Instant>,
    pub mouse: HashMap<MouseButton, Instant>,
    pub movement: (f64, f64),
}

impl Input {
    /// Create a new Input struct.
    pub fn new() -> Self {
        Self {
            is_focused: false,
            keys: HashMap::new(),
            mouse: HashMap::new(),
            movement: (0.0, 0.0),
        }
    }

    /// Get the duration a key has been pressed.
    pub fn get_key(&self, vk: KeyCode) -> f64 {
        if let Some(k) = self.keys.get(&vk) {
            return k.elapsed().as_millis() as f64 / 1000.0;
        }

        0.0
    }

    /// Get the duration a mouse button has been pressed.
    pub fn get_click(&self, button: MouseButton) -> f64 {
        if let Some(k) = self.mouse.get(&button) {
            return k.elapsed().as_millis() as f64 / 1000.0;
        }

        0.0
    }

    /// Process window event to update what keys or mouse buttons are pressed.
    pub fn handle(&mut self, state: &WindowState, event: &WindowEvent) {
        self.mouse_event(event);
        self.keyboard_event(state, event);
    }

    /// Update the mouse movement delta.
    pub fn mouse_delta(&mut self, delta: (f64, f64)) {
        self.movement = delta;
    }

    /// Get mouse button press state from event.
    fn mouse_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                self.mouse.entry(*button).or_insert(Instant::now());
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => {
                self.mouse.remove(button);
            }
            _ => {}
        }
    }

    /// Get key press state from event.
    fn keyboard_event(&mut self, window_state: &WindowState, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(v),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if v == &KeyCode::Escape {
                    let window = &window_state.window;
                    window
                        .set_cursor_grab(winit::window::CursorGrabMode::None)
                        .unwrap_or_default();
                    window.set_cursor_visible(true);
                }
                self.keys.entry(*v).or_insert(Instant::now());
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(v),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                self.keys.remove(v);
            }
            _ => {}
        }
    }
}
