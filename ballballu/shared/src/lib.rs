use serde::{Serialize, Deserialize};
use crate::objects::{PlayerSpec, Dot};

pub mod mechanics;
pub mod protocol;
pub mod objects;


/// Game Status Enum
/// Current state of the game
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameStatus {
    WaitingToStart,  // Start screen, waiting for players to press space
    Playing,         // Game is running
    GameOver,        // Game has ended
}

/// Game Constants
/// Core game constants used by both client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConstant {
    pub tick_interval_ms: u64,      // usually 20 ticks/sec = 50ms
    pub collide_size_fraction: f32, // size ratio needed to consume another player
    pub move_speed_base: f32,       // player default speed
    pub dot_radius: f32,            // constant dot size
}


/// Game Snapshot
/// Snapshot of game world sent from server → client every tick
/// Server sends:
///    - current state of all players
///    - all dots
///    - universal game constants
///    - current tick (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub tick: u64,
    pub status: GameStatus,  // Add game status
    pub players: Vec<PlayerSpec>,
    pub dots: Vec<Dot>,
    pub constants: GameConstant,
}
