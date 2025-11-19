use std::collections::HashMap;
use rand::Rng;

use shared::{
    GameConstant,
    GameSnapshot,
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
    pub players: HashMap<u64, PlayerSpec>,
    pub dots: HashMap<u64, Dot>,
    pub constants: GameConstant,
    // Phase 4: Store player inputs separately
    player_inputs: HashMap<u64, PlayerInput>,
    next_dot_id: u64,
}

impl GameState {
    pub fn new(constants: GameConstant) -> Self {
        let mut gs = Self {
            tick: 0,
            players: HashMap::new(),
            dots: HashMap::new(),
            constants,
            player_inputs: HashMap::new(),
            next_dot_id: 1,
        };
        // Phase 5: Initialize dots
        gs.spawn_initial_dots(150);
        gs
    }

    /// Phase 5: Spawn initial dots on the map
    fn spawn_initial_dots(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        let world_width = 2000.0;
        let world_height = 2000.0;
        for _ in 0..count {
            let id = self.next_dot_id;
            self.next_dot_id += 1;
            self.dots.insert(id, Dot {
                id,
                x: rng.gen_range(0.0..world_width),
                y: rng.gen_range(0.0..world_height),
                radius: self.constants.dot_radius,
                color: (255, 100, 100), // Red dots
            });
        }
    }

    /// Add new player when connected
    pub fn add_player(&mut self, id: u64) {
        let p = PlayerSpec {
            id,
            name: format!("Player{}", id),
            x: 400.0,
            y: 300.0,
            radius: 10.0,
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
        println!("GameState: Player {} added", id);
    }

    /// Remove player when disconnected
    pub fn remove_player(&mut self, id: u64) {
        self.players.remove(&id);
        self.player_inputs.remove(&id);
        println!("GameState: Player {} removed", id);
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
                // Phase 4: Store player input (legacy, kept for compatibility)
                if let Some(player_input) = self.player_inputs.get_mut(&id) {
                    player_input.dx = input.dx;
                    player_input.dy = input.dy;
                    println!("GameState: Player {} input: dx={}, dy={}", id, input.dx, input.dy);
                }
            }
            ClientMessage::Move { dx, dy, distance } => {
                // Store the move command to be processed next tick
                if let Some(player_input) = self.player_inputs.get_mut(&id) {
                    player_input.pending_move = Some((dx, dy, distance));
                    println!("GameState: Player {} queued move: dx={}, dy={}, distance={}", id, dx, dy, distance);
                }
            }
            ClientMessage::Quit => {
                println!("GameState: Player {} sent Quit", id);
                self.remove_player(id);
            }
        }
    }

    /// Phase 4: Get player input for movement
    pub fn get_player_input(&self, id: u64) -> (f32, f32) {
        self.player_inputs.get(&id)
            .map(|input| (input.dx, input.dy))
            .unwrap_or((0.0, 0.0))
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
            }
        }
    }

    /// Convert current world into snapshot
    pub fn to_snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            tick: self.tick,
            players: self.players.values().cloned().collect(),
            dots: self.dots.values().cloned().collect(),
            constants: self.constants.clone(),
        }
    }
}
