use macroquad::prelude::*;
use shared::protocol::ClientMessage;
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
        // WASD keys - use is_key_pressed for one-click-one-step (detects key press, not hold)
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            self.send_move(0.0, -1.0);
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            self.send_move(0.0, 1.0);
        }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            self.send_move(-1.0, 0.0);
        }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            self.send_move(1.0, 0.0);
        }

        // ESC to quit
        if is_key_pressed(KeyCode::Escape) {
            let _ = self.input_tx.send(ClientMessage::Quit);
            return true;
        }

        false
    }

    fn send_move(&self, dx: f32, dy: f32) {
        // Increment sequence number
        let _seq = SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed) + 1;

        // Send discrete move command (one step at a time)
        // Each key press = one movement action with fixed distance
        let distance = 50.0; // Fixed distance per step
        
        // Normalize
        let magnitude = ((dx * dx + dy * dy) as f32).sqrt();
        let normalized_dx = if magnitude > 0.0 { dx / magnitude } else { dx };
        let normalized_dy = if magnitude > 0.0 { dy / magnitude } else { dy };
        
        let msg = ClientMessage::Move { 
            dx: normalized_dx, 
            dy: normalized_dy, 
            distance 
        };
        let _ = self.input_tx.send(msg);
    }
}
