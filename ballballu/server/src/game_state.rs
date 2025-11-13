use std::collections::HashMap;

use shared::{
    PlayerSpec,
    Dot,
    GameConstant,
    GameSnapshot,
    protocol::ClientMessage,
};

pub struct GameState {
    pub tick: u64,
    pub players: HashMap<u64, PlayerSpec>,
    pub dots: HashMap<u64, Dot>,
    pub constants: GameConstant,
}

impl GameState {
    pub fn new(constants: GameConstant) -> Self {
        Self {
            tick: 0,
            players: HashMap::new(),
            dots: HashMap::new(),
            constants,
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
        };
        self.players.insert(id, p);
        println!("GameState: Player {} added", id);
    }

    /// Remove player when disconnected
    pub fn remove_player(&mut self, id: u64) {
        self.players.remove(&id);
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
            ClientMessage::Quit => {
                println!("GameState: Player {} sent Quit", id);
                self.remove_player(id);
            }
            _ => {
                // Input 留到 Phase 3 处理
                println!("GameState: Player {} sent non-Join/Quit message", id);
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
