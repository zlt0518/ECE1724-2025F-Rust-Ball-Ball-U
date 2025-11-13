use serde::{Serialize, Deserialize};

pub mod mechanics;
pub mod protocol;

// ==========================
//  Player + Dot Structures
// ==========================

/// Player state sent between server and clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSpec {
    pub id: u64,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub score: u32,
    pub speed: f32,          // movement speed
    pub sequence_number: u64 // counter for inputs / ordering
}

/// Food dots on the map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dot {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: (u8, u8, u8),
}


// ==========================
//      Game Constants
// ==========================

/// Core game constants used by both client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConstant {
    pub tick_interval_ms: u64,      // usually 20 ticks/sec = 50ms
    pub collide_size_fraction: f32, // size ratio needed to consume another player
    pub move_speed_base: f32,       // player default speed
    pub dot_radius: f32,            // constant dot size
}


// ==========================
//       Game Snapshot
// ==========================

/// Snapshot of game world sent from server → client every tick
///
/// Server sends:
///    - current state of all players
///    - all dots
///    - universal game constants
///    - current tick (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub tick: u64,
    pub players: Vec<PlayerSpec>,
    pub dots: Vec<Dot>,
    pub constants: GameConstant,
}
