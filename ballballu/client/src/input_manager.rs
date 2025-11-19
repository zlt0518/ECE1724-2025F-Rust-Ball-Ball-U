use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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

    /// Poll for keyboard input and convert to movement commands
    pub fn poll_input(&self) -> bool {
        if event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                // Only process key press events, ignore key release
                if key_event.kind == KeyEventKind::Press {
                    return self.handle_key(key_event);
                }
            }
        }
        false
    }

    fn handle_key(&self, key_event: KeyEvent) -> bool {
        let (dx, dy) = match key_event.code {
            // WASD keys
            KeyCode::Char('w') | KeyCode::Char('W') => (0.0, -1.0),
            KeyCode::Char('s') | KeyCode::Char('S') => (0.0, 1.0),
            KeyCode::Char('a') | KeyCode::Char('A') => (-1.0, 0.0),
            KeyCode::Char('d') | KeyCode::Char('D') => (1.0, 0.0),
            
            // Arrow keys
            KeyCode::Up => (0.0, -1.0),
            KeyCode::Down => (0.0, 1.0),
            KeyCode::Left => (-1.0, 0.0),
            KeyCode::Right => (1.0, 0.0),
            
            // Escape to quit
            KeyCode::Esc => {
                let _ = self.input_tx.send(ClientMessage::Quit);
                return true; // Signal to exit
            }
            
            _ => return false, // Ignore other keys
        };

        // Normalize diagonal movement
        let magnitude = ((dx * dx + dy * dy) as f32).sqrt();
        let normalized_dx = if magnitude > 0.0 { dx / magnitude } else { 0.0 };
        let normalized_dy = if magnitude > 0.0 { dy / magnitude } else { 0.0 };

        // Increment sequence number
        let seq = SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed) + 1;

        // Send input message to server
        let input = UserInput {
            dx: normalized_dx,
            dy: normalized_dy,
            sequence_number: seq,
        };

        let msg = ClientMessage::Input { input };
        let _ = self.input_tx.send(msg);
        false
    }
}

