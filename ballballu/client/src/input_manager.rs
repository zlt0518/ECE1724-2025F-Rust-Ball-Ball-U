use macroquad::prelude::*;
use shared::protocol::{ClientMessage, UserInput};
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE_NUMBER: AtomicU64 = AtomicU64::new(0);

pub struct InputManager {
    input_tx: mpsc::UnboundedSender<ClientMessage>,
}

impl InputManager {
    pub fn new(input_tx: mpsc::UnboundedSender<ClientMessage>) -> Self {
        Self { input_tx }
    }

    /// Poll for keyboard input and send to server
    /// Returns true if the application should exit
    pub fn poll_input(&self) -> bool {
        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut has_input = false;

        // WASD keys
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            dy = -1.0;
            has_input = true;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            dy = 1.0;
            has_input = true;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dx = -1.0;
            has_input = true;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dx = 1.0;
            has_input = true;
        }

        // ESC to quit
        if is_key_pressed(KeyCode::Escape) {
            let _ = self.input_tx.send(ClientMessage::Quit);
            return true;
        }

        if has_input {
            // Normalize diagonal movement
            let magnitude = ((dx * dx + dy * dy) as f32).sqrt();
            let normalized_dx = if magnitude > 0.0 { dx / magnitude } else { 0.0 };
            let normalized_dy = if magnitude > 0.0 { dy / magnitude } else { 0.0 };

            // Increment sequence number
            let seq = SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed) + 1;

            // Send continuous input message to server
            // Server will handle speed calculation based on player's score using mechanics
            let input = UserInput {
                dx: normalized_dx,
                dy: normalized_dy,
                sequence_number: seq,
            };

            let msg = ClientMessage::Input { input };
            let _ = self.input_tx.send(msg);
        }

        false
    }
}
