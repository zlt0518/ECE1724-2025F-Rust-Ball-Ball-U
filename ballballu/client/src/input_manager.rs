use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use shared::protocol::ClientMessage;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::render_manager::RenderManager;

pub struct InputManager {
    input_tx: mpsc::UnboundedSender<ClientMessage>,
    render: Arc<Mutex<RenderManager>>,
}

impl InputManager {
    pub fn new(
        input_tx: mpsc::UnboundedSender<ClientMessage>,
        render: Arc<Mutex<RenderManager>>,
    ) -> Self {
        Self { input_tx, render }
    }

    /// Poll for keyboard input and convert to movement commands
    pub fn poll_input(&self) -> bool {
        let mut should_exit = false;

        while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                if key_event.kind == KeyEventKind::Press {
                    if self.handle_key(key_event) {
                        should_exit = true;
                    }
                }
            } else {
                break;
            }
        }

        should_exit
    }

    fn handle_key(&self, key_event: KeyEvent) -> bool {
        let (dx, dy): (f32, f32) = match key_event.code {
            // WASD
            KeyCode::Char('w') | KeyCode::Char('W') => (0.0, -1.0),
            KeyCode::Char('s') | KeyCode::Char('S') => (0.0, 1.0),
            KeyCode::Char('a') | KeyCode::Char('A') => (-1.0, 0.0),
            KeyCode::Char('d') | KeyCode::Char('D') => (1.0, 0.0),

            // Arrows
            KeyCode::Up => (0.0, -1.0),
            KeyCode::Down => (0.0, 1.0),
            KeyCode::Left => (-1.0, 0.0),
            KeyCode::Right => (1.0, 0.0),

            // Quit
            KeyCode::Esc => {
                let _ = self.input_tx.send(ClientMessage::Quit);
                return true;
            }

            _ => return false,
        };

        let magnitude = (dx * dx + dy * dy).sqrt();
        let normalized_dx = if magnitude > 0.0 { dx / magnitude } else { 0.0 };
        let normalized_dy = if magnitude > 0.0 { dy / magnitude } else { 0.0 };

        // Distance = EXACTLY ONE SCREEN CELL
        let distance = {
            let mut rm = self.render.lock().unwrap();
            rm.cell_distance(dx, dy)
        };

        let msg = ClientMessage::Move {
            dx: normalized_dx,
            dy: normalized_dy,
            distance,
        };

        let _ = self.input_tx.send(msg);
        false
    }
}