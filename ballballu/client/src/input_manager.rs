use macroquad::prelude::*;
use macroquad::prelude::{is_key_pressed,is_key_down, KeyCode};
use shared::protocol::ClientMessage;
use tokio::sync::mpsc;

pub struct InputManager {
    input_tx: mpsc::UnboundedSender<ClientMessage>,
}

impl InputManager {
    pub fn new(input_tx: mpsc::UnboundedSender<ClientMessage>) -> Self {
        Self { input_tx }
    }

    /// Poll for keyboard input and send a single-step Move to the server.
    /// `player_radius` is used to scale the step distance; if `None`, a default
    /// base distance is used. Returns (should_exit, enter_pressed).
    pub fn poll_input(&self, player_radius: Option<f32>) -> (bool, bool) {
        // Enter to send Ready message
        if is_key_pressed(KeyCode::Enter) {
            let _ = self.input_tx.send(ClientMessage::Ready);
            return (false, true);
        }

        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        let mut has_press = false;

        // Use key down for discrete movement
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            dy -= 1.0;
            has_press = true;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            dy += 1.0;
            has_press = true;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dx -= 1.0;
            has_press = true;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dx += 1.0;
            has_press = true;
        }

        // ESC to quit
        if is_key_pressed(KeyCode::Escape) {
            let _ = self.input_tx.send(ClientMessage::Quit);
            return (true, false);
        }

        if has_press {
            // Normalize diagonal movement
            let mag = (dx * dx + dy * dy).sqrt();
            let ndx = if mag > 0.0 { dx / mag } else { 0.0 };
            let ndy = if mag > 0.0 { dy / mag } else { 0.0 };

            // Compute step distance based on player size (radius). If unknown,
            // fall back to a reasonable default.
            let base_radius = player_radius.unwrap_or(10.0);
            // Scale factor: one click moves approximately two radii.
            let distance = base_radius * 2.0;

            let msg = ClientMessage::Move { dx: ndx, dy: ndy, distance };
            let _ = self.input_tx.send(msg);
        }

        (false, false)
    }
}
