use serde::{Serialize, Deserialize};

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
    pub sequence_number: u64, // counter for inputs / ordering
    #[serde(default)]
    pub remaining_distance: f32, // distance left to move in current direction
    #[serde(default)]
    pub vx: f32,             // velocity x component
    #[serde(default)]
    pub vy: f32,             // velocity y component
}

/// Food dots on the map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dot {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: (u8, u8, u8),
    pub score: u32, // Score value of this dot (2, 5, or 10)
}
