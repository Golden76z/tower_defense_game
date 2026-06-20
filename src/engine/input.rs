//! Centralised keyboard and mouse input state.
//!
//! [`InputState`] tracks which keys and mouse buttons are currently held, which
//! were pressed *this frame* (edge-triggered), and the latest mouse position.
//! Feed it window events as they arrive, query it from game logic, then call
//! [`InputState::clear_frame_state`] once at the end of each frame to reset the
//! edge-triggered ("just pressed") sets.
//!
//! Note: the original issue referenced winit's `VirtualKeyCode`, but this
//! project targets winit 0.30 where that type is [`winit::keyboard::KeyCode`].

use std::collections::HashSet;

use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

/// Snapshot of keyboard and mouse state, updated from window events.
#[derive(Debug, Default, Clone)]
pub struct InputState {
    /// Keys currently held down.
    keys_pressed: HashSet<KeyCode>,
    /// Keys that transitioned to "down" this frame (edge-triggered).
    keys_just_pressed: HashSet<KeyCode>,
    /// Mouse position in physical pixels, origin top-left.
    mouse_position: (f32, f32),
    /// Mouse buttons currently held down.
    mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons that transitioned to "down" this frame (edge-triggered).
    mouse_buttons_just_pressed: HashSet<MouseButton>,
}

impl InputState {
    /// Creates an empty input state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds a key event. The OS sends repeated `Pressed` events while a key is
    /// held; "just pressed" only fires on the initial press (the key wasn't
    /// already down), so it triggers exactly once per physical press.
    pub fn process_key(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                // `insert` returns false if the key was already present, which
                // means this is an auto-repeat — don't re-fire just_pressed.
                if self.keys_pressed.insert(key) {
                    self.keys_just_pressed.insert(key);
                }
            }
            ElementState::Released => {
                self.keys_pressed.remove(&key);
            }
        }
    }

    /// Feeds a mouse button event.
    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.mouse_buttons.insert(button) {
                    self.mouse_buttons_just_pressed.insert(button);
                }
            }
            ElementState::Released => {
                self.mouse_buttons.remove(&button);
            }
        }
    }

    /// Updates the tracked mouse position (physical pixels, origin top-left).
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    /// Returns `true` while `key` is held down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Returns `true` only on the frame `key` was first pressed.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// Returns `true` while `button` is held down.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Returns `true` only on the frame `button` was first pressed.
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    /// The latest mouse position in physical pixels.
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Clears the edge-triggered ("just pressed") state. Call once at the end
    /// of every frame, after game logic has read it. Held state (`keys_pressed`,
    /// `mouse_buttons`) and the mouse position are preserved across frames.
    pub fn clear_frame_state(&mut self) {
        self.keys_just_pressed.clear();
        self.mouse_buttons_just_pressed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_press_and_release_detection() {
        let mut input = InputState::new();
        assert!(!input.is_key_pressed(KeyCode::KeyW));

        input.process_key(KeyCode::KeyW, ElementState::Pressed);
        assert!(input.is_key_pressed(KeyCode::KeyW));

        input.process_key(KeyCode::KeyW, ElementState::Released);
        assert!(!input.is_key_pressed(KeyCode::KeyW));
    }

    #[test]
    fn just_pressed_triggers_once_per_press() {
        let mut input = InputState::new();

        input.process_key(KeyCode::Space, ElementState::Pressed);
        assert!(input.is_key_just_pressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));

        // A second `Pressed` in the same frame (auto-repeat) must not re-fire.
        input.process_key(KeyCode::Space, ElementState::Pressed);
        // Still just one "just pressed" entry; clearing removes it entirely.
        input.clear_frame_state();
        assert!(!input.is_key_just_pressed(KeyCode::Space));
        // The key is still considered held until released.
        assert!(input.is_key_pressed(KeyCode::Space));

        // Holding across frames doesn't re-trigger just_pressed.
        input.process_key(KeyCode::Space, ElementState::Pressed);
        assert!(!input.is_key_just_pressed(KeyCode::Space));

        // Release then press again does re-trigger.
        input.process_key(KeyCode::Space, ElementState::Released);
        input.clear_frame_state();
        input.process_key(KeyCode::Space, ElementState::Pressed);
        assert!(input.is_key_just_pressed(KeyCode::Space));
    }

    #[test]
    fn clear_frame_state_keeps_held_keys() {
        let mut input = InputState::new();
        input.process_key(KeyCode::KeyA, ElementState::Pressed);
        input.clear_frame_state();

        assert!(!input.is_key_just_pressed(KeyCode::KeyA));
        assert!(input.is_key_pressed(KeyCode::KeyA));
    }

    #[test]
    fn mouse_position_updates() {
        let mut input = InputState::new();
        assert_eq!(input.mouse_position(), (0.0, 0.0));

        input.set_mouse_position(123.5, 456.0);
        assert_eq!(input.mouse_position(), (123.5, 456.0));

        input.set_mouse_position(10.0, 20.0);
        assert_eq!(input.mouse_position(), (10.0, 20.0));
    }

    #[test]
    fn mouse_button_press_release_and_just_pressed() {
        let mut input = InputState::new();
        assert!(!input.is_mouse_button_pressed(MouseButton::Left));

        input.process_mouse_button(MouseButton::Left, ElementState::Pressed);
        assert!(input.is_mouse_button_pressed(MouseButton::Left));
        assert!(input.is_mouse_button_just_pressed(MouseButton::Left));

        input.clear_frame_state();
        assert!(input.is_mouse_button_pressed(MouseButton::Left));
        assert!(!input.is_mouse_button_just_pressed(MouseButton::Left));

        input.process_mouse_button(MouseButton::Left, ElementState::Released);
        assert!(!input.is_mouse_button_pressed(MouseButton::Left));
    }

    #[test]
    fn independent_keys_do_not_interfere() {
        let mut input = InputState::new();
        input.process_key(KeyCode::KeyW, ElementState::Pressed);
        input.process_key(KeyCode::KeyA, ElementState::Pressed);

        assert!(input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_pressed(KeyCode::KeyA));
        assert!(!input.is_key_pressed(KeyCode::KeyD));

        input.process_key(KeyCode::KeyW, ElementState::Released);
        assert!(!input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_pressed(KeyCode::KeyA));
    }
}
