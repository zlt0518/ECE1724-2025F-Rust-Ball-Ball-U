use std::collections::HashMap;
use rand::Rng;

use shared::{
    GameConstant,
    GameSnapshot,
    GameStatus,
    protocol::ClientMessage,
    objects::{PlayerSpec, Dot},
};

// Phase 4: Store player input direction
#[derive(Debug, Clone)]
struct PlayerInput {
    pub dx: f32,
    pub dy: f32,
    pub pending_move: Option<(f32, f32, f32)>, // (dx, dy, distance) for next move
}

pub struct GameState {
    pub tick: u64,
    pub status: GameStatus,  // Track current game status
    pub players: HashMap<u64, PlayerSpec>,
    pub dots: HashMap<u64, Dot>,
    pub constants: GameConstant,
    // Phase 4: Store player inputs separately
    player_inputs: HashMap<u64, PlayerInput>,
    ready_players: HashMap<u64, bool>,  // Track which players are ready to start
    next_dot_id: u64,
}

impl GameState {
    pub fn new(constants: GameConstant) -> Self {
        let mut gs = Self {
            tick: 0,
            status: GameStatus::WaitingToStart,  // Start in waiting state
            players: HashMap::new(),
            dots: HashMap::new(),
            constants,
            player_inputs: HashMap::new(),
            ready_players: HashMap::new(),
            next_dot_id: 1,
        };
        // Phase 5: Initialize dots
        gs.spawn_initial_dots(150);
        gs
    }

    /// Helper function to calculate distance between two points
    fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
    }

    /// Find an empty position that doesn't overlap with any players or dots
    /// Returns (x, y) if found, or None if couldn't find after max_attempts
    fn find_empty_position(
        &self,
        radius: f32,
        max_attempts: usize,
    ) -> Option<(f32, f32)> {
        let mut rng = rand::thread_rng();
        let world_width = 2000.0;
        let world_height = 2000.0;
        let min_x = radius;
        let max_x = world_width - radius;
        let min_y = radius;
        let max_y = world_height - radius;

        for _ in 0..max_attempts {
            let x = rng.gen_range(min_x..max_x);
            let y = rng.gen_range(min_y..max_y);

            // Check collision with players
            let mut collides = false;
            for player in self.players.values() {
                let d = Self::distance(x, y, player.x, player.y);
                if d < (radius + player.radius) {
                    collides = true;
                    break;
                }
            }

            if collides {
                continue;
            }

            // Check collision with dots
            for dot in self.dots.values() {
                let d = Self::distance(x, y, dot.x, dot.y);
                if d < (radius + dot.radius) {
                    collides = true;
                    break;
                }
            }

            if !collides {
                return Some((x, y));
            }
        }

        // Fallback: return center position if all attempts failed
        Some((world_width / 2.0, world_height / 2.0))
    }

    /// Phase 5: Spawn initial dots on the map
    /// Generates dots with three different score values: 2 (blue), 5 (yellow), 10 (red)
    fn spawn_initial_dots(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        
        // Dot configurations: (score, color, radius)
        let dot_configs = [
            (2, (100, 150, 255), 4.0),   // Blue, small
            (5, (255, 255, 100), 6.0),   // Yellow, medium
            (10, (255, 100, 100), 8.0),  // Red, large
        ];
        
        for _ in 0..count {
            // Randomly select a dot type
            let config = dot_configs[rng.gen_range(0..dot_configs.len())];
            let (score, color, radius) = config;
            
            // Find empty position for this dot
            if let Some((x, y)) = self.find_empty_position(radius, 100) {
                let id = self.next_dot_id;
                self.next_dot_id += 1;
                self.dots.insert(id, Dot {
                    id,
                    x,
                    y,
                    radius,
                    color,
                    score,
                });
            } else {
                // If can't find empty position, still create dot at random position
                // (shouldn't happen often with 150 dots in 2000x2000 world)
                let world_width = 2000.0;
                let world_height = 2000.0;
                let id = self.next_dot_id;
                self.next_dot_id += 1;
                self.dots.insert(id, Dot {
                    id,
                    x: rng.gen_range(radius..(world_width - radius)),
                    y: rng.gen_range(radius..(world_height - radius)),
                    radius,
                    color,
                    score,
                });
            }
        }
    }

    /// Spawn a new dot at an empty position to maintain total dot count
    /// Returns true if successfully spawned, false otherwise
    pub fn spawn_new_dot(&mut self) -> bool {
        let mut rng = rand::thread_rng();
        
        // Dot configurations: (score, color, radius)
        let dot_configs = [
            (2, (100, 150, 255), 4.0),   // Blue, small
            (5, (255, 255, 100), 6.0),   // Yellow, medium
            (10, (255, 100, 100), 8.0),  // Red, large
        ];
        
        // Randomly select a dot type
        let config = dot_configs[rng.gen_range(0..dot_configs.len())];
        let (score, color, radius) = config;
        
        // Find empty position for this dot
        if let Some((x, y)) = self.find_empty_position(radius, 100) {
            let id = self.next_dot_id;
            self.next_dot_id += 1;
            self.dots.insert(id, Dot {
                id,
                x,
                y,
                radius,
                color,
                score,
            });
            true
        } else {
            // If can't find empty position, try a few more times with random positions
            let world_width = 2000.0;
            let world_height = 2000.0;
            for _ in 0..10 {
                let x = rng.gen_range(radius..(world_width - radius));
                let y = rng.gen_range(radius..(world_height - radius));
                
                // Quick check if position is empty
                let mut collides = false;
                for player in self.players.values() {
                    let d = Self::distance(x, y, player.x, player.y);
                    if d < (radius + player.radius) {
                        collides = true;
                        break;
                    }
                }
                if !collides {
                    for dot in self.dots.values() {
                        let d = Self::distance(x, y, dot.x, dot.y);
                        if d < (radius + dot.radius) {
                            collides = true;
                            break;
                        }
                    }
                }
                
                if !collides {
                    let id = self.next_dot_id;
                    self.next_dot_id += 1;
                    self.dots.insert(id, Dot {
                        id,
                        x,
                        y,
                        radius,
                        color,
                        score,
                    });
                    return true;
                }
            }
            false
        }
    }

    /// Add new player when connected
    pub fn add_player(&mut self, id: u64) {
        let base_radius = 10.0;
        let (x, y) = self.find_empty_position(base_radius, 100)
            .unwrap_or((1000.0, 1000.0)); // Fallback to center if all attempts fail
        
        let p = PlayerSpec {
            id,
            name: format!("Player{}", id),
            x,
            y,
            radius: base_radius,
            speed: self.constants.move_speed_base,
            score: 0,
            sequence_number: 0,
            remaining_distance: 0.0,
            vx: 0.0,
            vy: 0.0,
        };
        self.players.insert(id, p);
        // Phase 4: Initialize input to zero
        self.player_inputs.insert(id, PlayerInput { dx: 0.0, dy: 0.0, pending_move: None });
        // Mark player as not ready (must press space to start)
        self.ready_players.insert(id, false);
        println!("GameState: Player {} added at ({}, {})", id, x, y);
    }

    /// Remove player when disconnected
    pub fn remove_player(&mut self, id: u64) {
        self.players.remove(&id);
        self.player_inputs.remove(&id);
        self.ready_players.remove(&id);
        println!("GameState: Player {} removed", id);
    }

    /// Respawn a player after being eaten (resets to initial state at random position)
    pub fn respawn_player(&mut self, id: u64) {
        let base_radius = 10.0;
        let (x, y) = self.find_empty_position(base_radius, 100)
            .unwrap_or((1000.0, 1000.0)); // Fallback to center if all attempts fail
        
        if let Some(player) = self.players.get_mut(&id) {
            player.x = x;
            player.y = y;
            player.radius = base_radius;
            player.score = 0;
            player.speed = self.constants.move_speed_base;
            player.remaining_distance = 0.0;
            player.vx = 0.0;
            player.vy = 0.0;
            println!("GameState: Player {} respawned at ({}, {})", id, x, y);
        }
        
        // Reset input state
        if let Some(player_input) = self.player_inputs.get_mut(&id) {
            player_input.dx = 0.0;
            player_input.dy = 0.0;
            player_input.pending_move = None;
        }
    }

    /// Handle JSON from Client
    pub fn handle_message(&mut self, id: u64, msg: ClientMessage) {
        match msg {
            ClientMessage::Join { name } => {
                if let Some(p) = self.players.get_mut(&id) {
                    p.name = name.clone();
                }
                println!("GameState: Player {} set name to {}", id, name);
            }
            ClientMessage::Input { input } => {
                // Input message not used in current implementation
                println!("GameState: Player {} sent deprecated Input message: dx={}, dy={}", 
                    id, input.dx, input.dy);
            }
            ClientMessage::Move { dx, dy, distance } => {
                // Store the move command to be processed next tick
                if let Some(player_input) = self.player_inputs.get_mut(&id) {
                    player_input.pending_move = Some((dx, dy, distance));
                    println!("GameState: Player {} queued move: dx={}, dy={}, distance={}", id, dx, dy, distance);
                }
            }
            ClientMessage::Ready => {
                // Mark player as ready to start
                self.ready_players.insert(id, true);
                println!("GameState: Player {} is ready", id);
                
                // Check if all players are ready (and at least 1 player connected)
                if self.all_players_ready() && !self.players.is_empty() {
                    self.status = GameStatus::Playing;
                    println!("GameState: All players ready! Starting game!");
                }
            }
            ClientMessage::Quit => {
                // Quit is now handled in websocket_manager, this should not be reached
                println!("GameState: Player {} sent Quit (should be handled by websocket_manager)", id);
            }
        }
    }

    /// Check if all connected players are ready
    pub fn all_players_ready(&self) -> bool {
        if self.players.is_empty() {
            return false;
        }
        self.players.keys().all(|id| {
            self.ready_players.get(id).copied().unwrap_or(false)
        })
    }

    /// Apply pending moves to players
    pub fn apply_pending_moves(&mut self) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        
        for id in player_ids {
            if let Some(player_input) = self.player_inputs.get_mut(&id) {
                if let Some((dx, dy, distance)) = player_input.pending_move.take() {
                    // Only start a new move if not currently moving
                    if let Some(player) = self.players.get_mut(&id) {
                        if player.remaining_distance <= 0.0 && distance > 0.0 {
                            let mag = (dx * dx + dy * dy).sqrt();
                            if mag > 0.0 {
                                let speed = player.speed;
                                player.vx = (dx / mag) * speed;
                                player.vy = (dy / mag) * speed;
                                player.remaining_distance = distance;
                            }
                        }
                    }
                }
                
                // Clear stored continuous input (for one-click-one-step model)
                player_input.dx = 0.0;
                player_input.dy = 0.0;
            }
        }
    }

    /// Convert current world into snapshot
    pub fn to_snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            tick: self.tick,
            status: self.status,  // Include game status
            players: self.players.values().cloned().collect(),
            dots: self.dots.values().cloned().collect(),
            constants: self.constants.clone(),
        }
    }
}
